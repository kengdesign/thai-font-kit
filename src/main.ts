import { invoke, Channel } from '@tauri-apps/api/core';
import fallbackCatalog from '../core/catalog.json';
import './style.css';
import appIcon from '../app-icon.svg';

type FontFile = { filename: string; size: number; weight: number; italic: boolean };
type Family = { id: string; name: string; description: string; category: string; license: string; files: FontFile[] };
type Status = { id: string; installed: number; total: number; changed: number; managed: number };
type Outcome = { count: number; warning: string };
const native = '__TAURI_INTERNALS__' in window;
let families: Family[] = [];
let selected: Family;
let statuses: Status[] = [];
let filter = 'all';
let category = 'all';
let connected = false;
let compact = false;
function readIds(key: string): string[] {
  try { const value: unknown = JSON.parse(localStorage.getItem(key) || '[]'); return Array.isArray(value) ? value.filter((id): id is string => typeof id === 'string' && fallbackCatalog.families.some(f => f.id === id)).slice(0, 34) : []; } catch { return []; }
}
let favorites = new Set(readIds('tfk-favorites'));
let recent = readIds('tfk-recent');
function saveIds(key: string, ids: string[]) { try { localStorage.setItem(key, JSON.stringify(ids.slice(0, 34))); } catch { message('บันทึกรายการส่วนตัวไม่ได้ พื้นที่จัดเก็บอาจเต็ม', true); } }
function remember(id: string) { recent = [id, ...recent.filter(x => x !== id)].slice(0, 12); saveIds('tfk-recent', recent); }

let busy = false;
let previewBusy = false;
let bridgeBusy = false;
let workTimer: ReturnType<typeof setInterval> | undefined;
let loaded = new Set<string>();
let previewSequence = 0;
let activeFace: FontFace | undefined;
let previewError = '';
function work(text: string, done?: number, total?: number) {
  if (el('work-status').hidden) {
    const start = Date.now(); el('work-elapsed').textContent = '0 วินาที';
    clearInterval(workTimer);
    workTimer = setInterval(() => { el('work-elapsed').textContent = `${Math.floor((Date.now() - start) / 1000)} วินาที`; }, 1000);
  }
  el('work-status').hidden = false;
  el('work-text').textContent = text;
  const bar = el<HTMLProgressElement>('work-progress');
  if (total) { bar.max = total; bar.value = done ?? 0; } else { bar.removeAttribute('value'); }
}

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <aside class="sidebar">
    <div class="brand"><img class="brand-icon" src="${appIcon}" alt="ไอคอน Thai Font Kit"><div>Thai Font Kit<small>ฟอนต์ไทย พร้อมใช้</small></div></div>
    <div class="side-label">คลังฟอนต์</div>
    <nav aria-label="หมวดฟอนต์">
      <button class="nav active" data-filter="all"><span>กข</span>ฟอนต์ทั้งหมด <b id="all-count">—</b></button>
      <button class="nav" data-filter="installed"><span>✓</span>ติดตั้งผ่านแอปแล้ว <b id="installed-count">—</b></button>
      <button class="nav" data-filter="favorites"><span>★</span>รายการโปรด</button>
      <button class="nav" data-filter="recent"><span>◷</span>ใช้กับ PhotoCraft ล่าสุด</button>
    </nav>
    <div class="side-note"><span class="note-icon">↙</span><strong>เลือก • ทดลอง • ติดตั้ง</strong><p>ดาวน์โหลดเฉพาะฟอนต์ที่เลือก<br>ติดตั้งแล้วใช้งานออฟไลน์ได้</p></div>
    <div class="side-bottom">OPEN SOURCE · v0.3.1 alpha<br><span>ฟอนต์จาก Google Fonts</span></div>
  </aside>
  <main>
    <header><div class="eyebrow">MADE FOR THAI CREATORS</div><h1>ฟอนต์ไทยที่ใช่<br><span>พร้อมใช้ในเครื่องคุณ</span></h1><p>เลือกฟอนต์สำหรับงานชิ้นต่อไป ทดลองข้อความ แล้วติดตั้งได้ในไม่กี่คลิก</p><span class="badge">ใช้งานฟรี · ไม่ต้องสมัครสมาชิก</span></header>
    <div id="notice" class="notice" role="status" aria-live="polite">กำลังเปิดคลังฟอนต์…</div>
    <section class="bridge-panel" aria-label="เชื่อมต่อ PhotoCraft">
      <div><strong>ใช้ฟอนต์ใน PhotoCraft <small>ทดลอง</small></strong><p id="bridge-status" role="status">บันทึกงานและปิด PhotoCraft เดิมก่อนเปิดผ่านปุ่มนี้ครั้งแรก</p></div>
      <div class="bridge-actions"><button id="compact" class="secondary">แผงเล็ก · อยู่ด้านหน้า</button><button id="bridge-open" class="secondary">เปิด PhotoCraft และเชื่อมต่อ</button><button id="bridge-check" class="text-button">เชื่อมต่ออีกครั้ง</button><button id="bridge-select" class="text-button">เลือกโปรแกรมใหม่</button></div>
      <details><summary>วิธีใช้ตัวเชื่อม</summary><p>ครั้งแรกเลือกไฟล์ PhotoCraft ที่ติดตั้งไว้ แอปจะเปิดให้พร้อมตัวเชื่อมในเครื่อง จากนั้นเปิดเอกสารและเลือกเลเยอร์ข้อความ กลับมาเลือกฟอนต์แล้วกด “ใช้ใน PhotoCraft” การเปลี่ยนฟอนต์ยังแก้ไขข้อความต่อได้ และย้อนกลับได้ใน PhotoCraft<br>หากเพิ่งติดตั้งฟอนต์ ให้บันทึกงาน ปิด PhotoCraft แล้วเปิดผ่านปุ่มนี้ใหม่ ตัวเชื่อมยังไม่ใช่แท็บที่ฝังใน PhotoCraft และไม่บันทึกเอกสารให้อัตโนมัติ</p></details>
    </section>
    <div class="workspace">
      <section class="library" aria-label="รายการฟอนต์">
        <label class="search"><span>⌕</span><input id="search" type="search" placeholder="ค้นหาชื่อฟอนต์ เช่น Sarabun" aria-label="ค้นหาฟอนต์"></label>
        <label class="category-label">หมวดฟอนต์ <select id="category"><option value="all">ทุกหมวด</option><option value="SANS_SERIF">Sans Serif</option><option value="SERIF">Serif</option><option value="HANDWRITING">ลายมือ</option><option value="DISPLAY">ดิสเพลย์ / หัวเรื่อง</option></select></label>
        <div class="list-heading"><span id="list-title">ฟอนต์ทั้งหมด</span><span id="list-count"></span></div>
        <div id="font-list"></div>
      </section>
      <section class="detail" aria-label="ทดลองฟอนต์">
        <div class="detail-top"><div><span class="eyebrow">FONT PREVIEW</span><h2 id="family-name"></h2><button id="favorite" class="favorite" aria-pressed="false">☆ เพิ่มรายการโปรด</button><p id="family-description"></p></div><span class="file-tag" id="file-tag"></span></div>
        <div class="preview-toolbar"><label for="size">ขนาดตัวอักษร</label><input type="range" id="size" min="20" max="72" value="40"><output id="size-output">40 px</output></div>
        <label for="sample" class="sample-label">ทดลองพิมพ์ข้อความของคุณ</label>
        <textarea id="sample" spellcheck="false" rows="5">สร้างสรรค์ได้ทุกวัน\nด้วยฟอนต์ภาษาไทย\nก้าวใหม่ของคนทำงานออกแบบ</textarea>
        <div class="preview-foot"><span id="preview-state">กดทดลองฟอนต์เพื่อดาวน์โหลดตัวอย่างจริง</span><button id="preview" class="text-button">ทดลองฟอนต์ ↓</button></div>
        <div class="glyphs" aria-label="ชุดอักษรตัวอย่าง">ก ข ค ง จ · สระอิ สระอุ · น้ำ ผู้ใหญ่ เก้าอี้<br><span>ABCDEFGHIJKLMNOPQRSTUVWXYZ<br>0123456789 · ฿ ! ? @ #</span></div>
        <div class="install-info"><span id="install-state"></span><small id="download-size"></small></div>
        <div id="work-status" class="work-status" role="status" aria-live="polite" hidden><span class="spinner" aria-hidden="true"></span><strong id="work-text"></strong><small id="work-elapsed"></small><progress id="work-progress" aria-label="ความคืบหน้า"></progress></div>
        <div class="actions"><button id="install" class="primary">ติดตั้งฟอนต์นี้ <span>↓</span></button><button id="uninstall" class="secondary">ถอนฟอนต์</button></div>
        <button id="apply-photocraft" class="primary apply-button" disabled>ใช้ใน PhotoCraft →</button><p id="apply-hint" class="apply-hint">เชื่อมต่อ PhotoCraft เพื่อใช้กับเลเยอร์ข้อความที่เลือก</p>
        <p class="restart-note">หลังติดตั้ง ให้ปิดแล้วเปิดโปรแกรมออกแบบอีกครั้งเพื่อโหลดรายชื่อฟอนต์</p>
        <button id="license" class="license-button">ดูแหล่งที่มาและสัญญาอนุญาต OFL ↗</button>
      </section>
    </div>
    <footer>ใช้ร่วมกับโปรแกรมที่รองรับฟอนต์ระบบ · การแสดงภาษาไทยขึ้นอยู่กับระบบข้อความของแต่ละโปรแกรม</footer>
  </main>
  <dialog id="confirm-dialog"><h2 id="confirm-title"></h2><p id="confirm-body"></p><div class="dialog-actions"><button id="cancel" class="secondary">ยกเลิก</button><button id="proceed" class="primary">ยืนยัน</button></div></dialog>
  <dialog id="license-dialog"><h2>แหล่งที่มาและสิทธิ์การใช้งาน</h2><p id="source"></p><pre id="license-text"></pre><button id="close-license" class="primary">ปิด</button></dialog>
`;
function el<T extends HTMLElement = HTMLElement>(id: string): T { return document.getElementById(id) as T; }
function message(text: string, error = false) { el('notice').textContent = text; el('notice').classList.toggle('error', error); }
function status(f: Family) { return statuses.find(s => s.id === f.id) ?? { id: f.id, installed: 0, total: f.files.length, changed: 0, managed: 0 }; }
function size(bytes: number) { return `${(bytes / 1_000_000).toFixed(1)} MB`; }
function setBusy(value: boolean) {
  busy = value;
  document.querySelectorAll<HTMLButtonElement>('.nav, .font-card, #preview, #install, #uninstall, #apply-photocraft').forEach(b => { b.disabled = value; });
  el<HTMLInputElement>('search').disabled = value;
  el('app').setAttribute('aria-busy', String(value));
  if (!value) { endWork(); detail(); }
}
function endWork() { el('work-status').hidden = true; clearInterval(workTimer); workTimer = undefined; }
function renderList() {
  const query = el<HTMLInputElement>('search').value.trim().toLowerCase();
  let visible = families.filter(f => (category === 'all' || f.category === category) && (filter !== 'favorites' || favorites.has(f.id)) && (filter !== 'recent' || recent.includes(f.id)) && (filter !== 'installed' || status(f).managed > 0) && `${f.name} ${f.description}`.toLowerCase().includes(query));
  if (filter === 'recent') visible.sort((a, b) => recent.indexOf(a.id) - recent.indexOf(b.id));
  el('font-list').replaceChildren();
  for (const f of visible) {
    const button = document.createElement('button');
    button.className = `font-card ${f.id === selected?.id ? 'selected' : ''}`;
    button.disabled = busy;
    button.setAttribute('aria-pressed', String(f.id === selected?.id));
    const title = document.createElement('strong'); title.textContent = `${favorites.has(f.id) ? '★ ' : ''}${f.name}`;
    const desc = document.createElement('span'); desc.textContent = f.description;
    const meta = document.createElement('small');
    const s = status(f);
    meta.textContent = s.changed ? 'พบไฟล์ที่ถูกแก้ไข' : s.installed === s.total ? '✓ ติดตั้งแล้ว' : s.installed ? `ติดตั้ง ${s.installed}/${s.total} ไฟล์` : `${f.files.length} ไฟล์ · ${size(f.files.reduce((n, file) => n + file.size, 0))}`;
    button.append(title, desc, meta);
    button.onclick = () => { selected = f; previewSequence++; previewError = ''; renderList(); detail(); if (native) void preview(); };
    el('font-list').append(button);
  }
  if (!visible.length) { const p = document.createElement('p'); p.className = 'empty'; p.textContent = 'ยังไม่มีฟอนต์ในรายการนี้'; el('font-list').append(p); }
  el('list-count').textContent = `${visible.length} แบบ`;
  el('all-count').textContent = String(families.length);
  el('installed-count').textContent = String(statuses.filter(s => s.installed > 0).length);
}
function detail() {
  if (!selected) return;
  const s = status(selected);
  el('family-name').textContent = selected.name;
  el('favorite').textContent = favorites.has(selected.id) ? '★ บันทึกในรายการโปรดแล้ว' : '☆ เพิ่มรายการโปรด';
  el('favorite').setAttribute('aria-pressed', String(favorites.has(selected.id)));
  el<HTMLButtonElement>('apply-photocraft').disabled = busy || previewBusy || !native || !connected;
  el('apply-hint').textContent = connected ? 'เลือกเลเยอร์ข้อความใน PhotoCraft ก่อนกดใช้ · ย้อนกลับได้ใน PhotoCraft' : 'เปิด PhotoCraft ผ่านปุ่มด้านบนเพื่อเชื่อมต่อ';

  el('family-description').textContent = selected.description;
  el('file-tag').textContent = `${selected.files.length} ไฟล์ TTF`;
  const actual = loaded.has(selected.id);
  const fontFamily = actual ? `"preview-${selected.id}"` : '"Thai Font Kit UI", sans-serif';
  el('sample').style.fontFamily = fontFamily;
  document.querySelector<HTMLElement>('.glyphs')!.style.fontFamily = fontFamily;
  el('preview-state').textContent = actual ? `แสดงด้วย ${selected.name} จริง` : previewError || (native ? 'กำลังเตรียมตัวอย่างฟอนต์จริง…' : 'ตัวอย่างหน้าจอ — เปิดแอปเดสก์ท็อปเพื่อโหลดฟอนต์จริง');
  el('preview').textContent = actual ? 'โหลดตัวอย่างแล้ว ✓' : 'ลองโหลดตัวอย่างอีกครั้ง ↻';
  el('install-state').textContent = s.changed ? 'มีไฟล์ที่ถูกแก้ไข แอปจะไม่ลบหรือเขียนทับไฟล์นั้น' : s.installed ? `ติดตั้งผ่านแอปแล้ว ${s.installed}/${s.total} ไฟล์` : 'ติดตั้งสำหรับบัญชีผู้ใช้ปัจจุบัน';
  el('download-size').textContent = `รวมทุกน้ำหนัก ${size(selected.files.reduce((n, f) => n + f.size, 0))}`;
  el('install').textContent = s.installed === s.total ? 'ตรวจซ่อมการติดตั้ง ↻' : 'ติดตั้งฟอนต์นี้ ↓';
  el<HTMLButtonElement>('install').disabled = busy || previewBusy || !native;
  el<HTMLButtonElement>('uninstall').disabled = busy || previewBusy || !native || !s.managed;
  el<HTMLButtonElement>('preview').disabled = busy || previewBusy || !native || actual;
}
async function refresh() { statuses = await invoke<Status[]>('font_status'); renderList(); detail(); }
async function preview() {
  const f = selected; const seq = ++previewSequence;
  if (busy || previewBusy || loaded.has(f.id)) return;
  previewBusy = true; detail(); previewError = '';
  el('sample').classList.add('preview-loading');
  el('preview-state').textContent = `กำลังโหลดตัวอย่าง ${f.name}…`;
  work(`กำลังโหลดตัวอย่าง ${f.name}…`);
  message(`กำลังโหลดตัวอย่าง ${f.name}…`);
  try {
    const bytes = await invoke<number[] | ArrayBuffer>('preview_font', { id: f.id });
    if (seq !== previewSequence) return;
    const face = await new FontFace(`preview-${f.id}`, new Uint8Array(bytes)).load();
    if (activeFace) document.fonts.delete(activeFace);
    activeFace = face; loaded.clear(); document.fonts.add(face); loaded.add(f.id);
    if (seq === previewSequence) message(`พร้อมทดลอง ${f.name} — ยังไม่ได้ติดตั้งฟอนต์ลงเครื่อง`);
  } catch (error) { if (seq !== previewSequence) return; previewError = `โหลดตัวอย่าง ${f.name} ไม่สำเร็จ — ยังแสดงฟอนต์หน้าจอ กดลองอีกครั้ง`; message(`${previewError}: ${String(error)}`, true); }
  finally { previewBusy = false; el('sample').classList.remove('preview-loading'); endWork(); detail(); if (selected.id !== f.id) void preview(); }
}
let pending: (() => void) | undefined;
function confirmAction(remove: boolean) {
  const f = selected;
  el('confirm-title').textContent = `${remove ? 'ถอน' : 'ติดตั้ง'} ${f.name}?`;
  el('confirm-body').textContent = remove ? 'แอปจะถอนเฉพาะไฟล์ที่เคยติดตั้งผ่าน Thai Font Kit เอกสารที่ใช้ฟอนต์นี้อาจแสดงต่างไปเมื่อเปิดใหม่' : `ดาวน์โหลดและติดตั้ง ${f.files.length} ไฟล์ รวม ${size(f.files.reduce((n, file) => n + file.size, 0))} สำหรับผู้ใช้คนนี้ ใช้ได้ในโปรแกรมที่อ่านฟอนต์ระบบ`;
  pending = () => { void changeFont(f, remove); };
  el<HTMLDialogElement>('confirm-dialog').showModal();
}
async function changeFont(f: Family, remove: boolean) {
  setBusy(true); message(`${remove ? 'กำลังถอน' : 'กำลังดาวน์โหลดและติดตั้ง'} ${f.name}… กรุณารอสักครู่`);
  work(`${remove ? 'กำลังถอน' : 'กำลังเตรียมติดตั้ง'} ${f.name}…`);
  el('install').textContent = remove ? 'กำลังถอนฟอนต์…' : 'กำลังติดตั้ง…';
  const onProgress = new Channel<{ phase: string; done: number; total: number }>();
  onProgress.onmessage = ({ phase, done, total }) => {
    const text = phase === 'download' ? `กำลังดาวน์โหลด ${f.name} · ไฟล์ ${done + 1}/${total}` : phase === 'install' ? `กำลังติดตั้ง ${f.name} · ไฟล์ ${done + 1}/${total}` : 'กำลังอัปเดตรายชื่อฟอนต์ในระบบ…';
    work(text, phase === 'install' ? total + done : phase === 'refresh' ? total * 2 : done, total * 2);
    message(text);
  };
  try {
    const result = await invoke<Outcome>(remove ? 'uninstall_font' : 'install_font', { id: f.id, onProgress });
    message(`${remove ? 'ถอน' : 'ติดตั้ง'} ${f.name} แล้ว ${result.count} ไฟล์${result.warning ? ` · ${result.warning}` : ' · ปิดแล้วเปิดโปรแกรมออกแบบอีกครั้ง'}`);
  } catch (error) { message(String(error), true); }
  finally {
    try { await refresh(); } catch (error) { message(String(error), true); }
    setBusy(false); renderList();
  }
}
el('search').oninput = renderList;
el('category').onchange = () => { category = el<HTMLSelectElement>('category').value; renderList(); };
el('favorite').onclick = () => { if (favorites.has(selected.id)) favorites.delete(selected.id); else favorites.add(selected.id); saveIds('tfk-favorites', [...favorites]); renderList(); detail(); };
async function bridgeAction(action: 'open' | 'select' | 'check') {
  if (bridgeBusy || !native) return;
  bridgeBusy = true; connected = false; detail(); document.querySelector('.bridge-panel')!.setAttribute('aria-busy', 'true');
  for (const id of ['bridge-open', 'bridge-select', 'bridge-check']) el<HTMLButtonElement>(id).disabled = true;
  el('bridge-status').textContent = action === 'check' ? 'กำลังตรวจการเชื่อมต่อ…' : 'กำลังเปิด PhotoCraft…';
  try {
    await invoke(action === 'check' ? 'photocraft_check' : 'photocraft_open', { select: action === 'select' });
    connected = true; el('bridge-status').textContent = '● เชื่อมต่อแล้ว · เปิดเอกสารและเลือกเลเยอร์ข้อความใน PhotoCraft';
  } catch (error) { connected = false; el('bridge-status').textContent = String(error); }
  finally { bridgeBusy = false; document.querySelector('.bridge-panel')!.setAttribute('aria-busy', 'false'); for (const id of ['bridge-open', 'bridge-select', 'bridge-check']) el<HTMLButtonElement>(id).disabled = false; detail(); }
}
el('compact').onclick = async () => {
  if (!native || busy) return;
  try { await invoke('companion_mode', { enabled: !compact }); compact = !compact; document.body.classList.toggle('compact', compact); el('compact').textContent = compact ? 'กลับหน้าจอเต็ม' : 'แผงเล็ก · อยู่ด้านหน้า'; }
  catch (error) { message(String(error), true); }
};
el('bridge-open').onclick = () => { void bridgeAction('open'); };
el('bridge-select').onclick = () => { void bridgeAction('select'); };
el('bridge-check').onclick = () => { void bridgeAction('check'); };
el('apply-photocraft').onclick = async () => {
  if (busy || previewBusy || !native || !connected) return;
  const f = selected; setBusy(true); work(`กำลังใช้ ${f.name} ใน PhotoCraft…`);
  try { await invoke('photocraft_apply', { id: f.id }); remember(f.id); message(`ใช้ ${f.name} กับเลเยอร์ข้อความใน PhotoCraft แล้ว · ย้อนกลับได้ใน PhotoCraft`); }
  catch (error) { message(String(error), true); }
  finally { setBusy(false); renderList(); }
};
el('size').oninput = () => { const value = el<HTMLInputElement>('size').value; el('sample').style.fontSize = `${value}px`; el('size-output').textContent = `${value} px`; };
el('preview').onclick = () => { void preview(); };
el('install').onclick = () => confirmAction(false);
el('uninstall').onclick = () => confirmAction(true);
el('cancel').onclick = () => { pending = undefined; el<HTMLDialogElement>('confirm-dialog').close(); };
el('proceed').onclick = () => { el<HTMLDialogElement>('confirm-dialog').close(); const action = pending; pending = undefined; action?.(); };
el('license').onclick = () => {
  el('source').textContent = `Google Fonts / ${selected.name} · ofl/${selected.id} · revision ${fallbackCatalog.revision}`;
  el('license-text').textContent = selected.license;
  el<HTMLDialogElement>('license-dialog').showModal();
};
el('close-license').onclick = () => el<HTMLDialogElement>('license-dialog').close();
document.querySelectorAll<HTMLButtonElement>('[data-filter]').forEach(button => button.onclick = () => {
  filter = button.dataset.filter!;
  document.querySelectorAll('.nav').forEach(b => b.classList.remove('active')); button.classList.add('active');
  el('list-title').textContent = ({ all: 'ฟอนต์ทั้งหมด', installed: 'ติดตั้งผ่านแอปแล้ว', favorites: 'รายการโปรด', recent: 'ใช้กับ PhotoCraft ล่าสุด' }[filter] || 'ฟอนต์ทั้งหมด'); renderList();
});
async function init() {
  try {
    families = native ? (await invoke<{ families: Family[] }>('catalog')).families : fallbackCatalog.families;
    selected = families[0];
    if (native) { await refresh(); await preview(); }
    else { for (const id of ['bridge-open', 'bridge-check', 'bridge-select', 'compact']) el<HTMLButtonElement>(id).disabled = true; message('ตัวอย่างหน้าจอ — การทดลองและติดตั้งฟอนต์ใช้งานได้ในแอปเดสก์ท็อป'); renderList(); detail(); }
  } catch (error) { message(`เปิดคลังฟอนต์ไม่สำเร็จ: ${String(error)}`, true); }
}
void init();
