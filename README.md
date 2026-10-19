# Thai Font Kit

แอปโอเพนซอร์สสำหรับเลือก ทดลอง และติดตั้ง Google Fonts ภาษาไทยลงเครื่อง

**สถานะ: 0.2.0 Alpha — ตัวติดตั้งผ่าน GitHub Actions ทั้ง Windows, macOS และ Linux; ยังต้องทดสอบการใช้งานจริงกับโปรแกรมออกแบบ**

[ดาวน์โหลดรุ่น Alpha](https://github.com/kengdesign/thai-font-kit/releases) · [ดูผล build](https://github.com/kengdesign/thai-font-kit/actions)

รองรับเป้าหมาย Windows x64, macOS Apple Silicon / Intel และ Linux x64 ใช้ Tauri 2 + Rust และ WebView ของระบบ ไม่มีบัญชีผู้ใช้ ไม่มี telemetry ไม่มีบริการทำงานเบื้องหลัง และไม่มีระบบอัปเดตอัตโนมัติในรุ่นแรก

เป็นแอปเสริมแยกต่างหาก ไม่ใช่ปลั๊กอินที่ติดตั้งภายใน ArtCraft และไม่มีความเกี่ยวข้องอย่างเป็นทางการกับ ArtCraft หรือ Google ยังไม่ได้รับรองความเข้ากันได้กับ PhotoCraft / VectorCraft / DesignCraft ต้องทดสอบระบบจัดวางภาษาไทยและการส่งออกของแต่ละแอป

## ทำอะไรได้แล้ว

- ค้นหาฟอนต์ภาษาไทยครบ 34 ตระกูล รวม 257 ไฟล์ ตามรายการ Google Fonts ดู [รายชื่อทั้งหมด](docs/FONTS.md)
- เพิ่มจากรุ่น 0.1 จำนวน 26 ตระกูล โดยคงข้อมูลฟอนต์เดิมให้จัดการต่อได้หลังอัปเดต
- ทดลองข้อความด้วยไฟล์ฟอนต์จริงใน WebView ก่อนติดตั้ง (โหลดเฉพาะไฟล์ตัวอย่าง)
- ติดตั้งทุกน้ำหนักในตระกูลที่เลือก สำหรับผู้ใช้ปัจจุบัน ไม่ขอสิทธิ์ admin
- ดาวน์โหลดจาก Google Fonts repository โดย pin commit และตรวจขนาด / SHA-256 / glyph ภาษาไทย
- จัดเก็บแคช เมื่อโหลดครบแล้วติดตั้งซ้ำได้แบบออฟไลน์
- ถอนเฉพาะไฟล์ที่มีบันทึกว่าแอปติดตั้งเอง พร้อมตรวจ checksum ก่อนลบ
- ปฏิเสธการเขียนทับไฟล์เดิมและการลบไฟล์ที่ถูกแก้ไข / symlink
- ป้องกันสองหน้าต่างติดตั้งพร้อมกัน และบันทึกรายการติดตั้งเพื่อกู้คืนงานที่ค้าง
- ดูสัญญาอนุญาต OFL และแหล่งที่มาในแอป

## ดาวน์โหลดและใช้งาน

เมื่อมี release ที่ผ่านการตรวจสอบแล้ว ดาวน์โหลดไฟล์ให้ตรงกับระบบจากหน้า Releases ของ repository นี้

| ระบบ | ไฟล์เป้าหมาย |
| --- | --- |
| Windows 10/11 x64 | ตัวติดตั้ง NSIS `.exe` แบบ current user (ต้องมี WebView2) |
| macOS 11+ Apple Silicon | `.dmg` สำหรับ aarch64 |
| macOS 11+ Intel | `.dmg` สำหรับ x86_64 |
| Linux x64 | `.deb` และ `.AppImage` (ต้องมี GTK / WebKitGTK 4.1 และแนะนำ fontconfig) |

เลือกฟอนต์ → ทดลองข้อความ → ติดตั้ง → ปิดและเปิดโปรแกรมออกแบบอีกครั้ง ฟอนต์ที่ติดตั้งใช้ได้ออฟไลน์ในแอปที่อ่านฟอนต์ของระบบ การวางสระ วรรณยุกต์ การตัดคำ และรูปแบบไฟล์ส่งออกยังขึ้นอยู่กับ text engine ของแอปนั้น

ตัวติดตั้ง alpha ยังไม่ได้เซ็น publisher certificate / notarize จึงอาจถูกระบบปฏิบัติการเตือน ไม่ควรเผยแพร่เป็น stable จนกว่าจะทดสอบและจัดการการเซ็นโปรแกรมเรียบร้อย

การถอนตัวโปรแกรมไม่ถอนฟอนต์อัตโนมัติ เพื่อรักษาเอกสารของผู้ใช้ หากต้องการถอนฟอนต์ ให้ทำผ่านแอปก่อนถอนโปรแกรม

## พัฒนาบนเครื่อง

ต้องมี Node.js 22, Rust stable และ [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) ของระบบนั้น

```sh
npm ci
npm run tauri dev
```

เปิดเฉพาะหน้าจอตัวอย่างใน browser ได้ด้วย `npm run dev` แต่จะปิดปุ่มทดลอง / ติดตั้ง เพราะไม่มี native backend

```sh
npm run build
cargo test --locked -p thai-font-kit-core
cargo fmt --all --check
npx playwright install chromium
npm test
```

`npm run tauri build` สร้าง installer สำหรับระบบที่ใช้ build การ build Windows / macOS ต้องใช้เครื่องหรือ GitHub runner ที่ตรงระบบ

## GitHub Actions / เผยแพร่

Push `main` หรือกด Actions → Test and build desktop installers → Run workflow เพื่อ build สี่เป้าหมาย ดาวน์โหลดไฟล์จาก Artifacts หลังงานสำเร็จ ผล build ทุกระบบต้องเป็นสีเขียวก่อนทดสอบบนเครื่องจริง

หลัง build จาก `main` ทุกเป้าหมายและการทดสอบหน้าจอสำเร็จ ระบบจะเผยแพร่ **Alpha prerelease สำหรับผู้ทดสอบ** พร้อม installers และ SHA256SUMS โดยใช้ tag `v<version>-alpha.<run number>` สามารถสร้าง version tag เองได้เช่นกัน ยังไม่ใช่ stable release

ก่อนเปลี่ยนเวอร์ชัน ให้อัปเดต `package.json`, `src-tauri/tauri.conf.json` และ Cargo packages พร้อม lockfiles และผลทดสอบบนเครื่องจริง

ดู [รายการตรวจรับ](docs/QA.md) และ [ข้อจำกัด / การออกแบบ](docs/ARCHITECTURE.md)

## อัปเดตรายการฟอนต์

```sh
python3 scripts/catalog.py
cargo test -p thai-font-kit-core
```

เครื่องมือนี้อ่าน METADATA.pb ของตระกูลที่กำหนด ดาวน์โหลดไฟล์และ OFL เพื่อสร้าง checksum ที่ commit แน่นอน ไม่ต้องใช้ Google API key รายการไม่ได้ sync อัตโนมัติ และรุ่นแรกยังไม่รองรับการย้าย receipt ข้ามการเปลี่ยนชื่อ / checksum ของฟอนต์ ดังนั้น **ห้ามเปลี่ยน catalog ใน release ใหม่โดยไม่มี migration** หาก checksum Anuphan เปลี่ยน ต้องอัปเดต test fixture และ OFL ให้ตรงกันด้วย

## สิทธิ์และความเป็นส่วนตัว

โค้ด MIT; ฟอนต์ยังคงอยู่ภายใต้ SIL Open Font License ของแต่ละโครงการ ข้อความ OFL ฝังอยู่ใน catalog และบันทึกลงโฟลเดอร์ข้อมูลแอปเมื่อมีการติดตั้ง ชุดทดสอบมี Anuphan ที่มาพร้อม OFL ของตนเอง

แอปส่งคำขอ HTTPS ไป `raw.githubusercontent.com` เฉพาะตอนดาวน์โหลดฟอนต์ที่ไม่มีในแคช ไม่ส่งข้อความที่ผู้ใช้ทดลองพิมพ์ ไม่ส่งรายการฟอนต์ และไม่ส่งไฟล์งานไปเซิร์ฟเวอร์
