//! Small authenticated, loopback-only client for PhotoCraft's JSON-line control protocol.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::time::{Duration, Instant};

pub const PORT: u16 = 17878;
pub struct Bridge(BufReader<TcpStream>, Instant);
impl Bridge {
    pub fn connect(token: &str) -> Result<Self, String> {
        Self::connect_for(token, Duration::from_secs(12))
    }
    pub fn connect_for(token: &str, budget: Duration) -> Result<Self, String> {
        Self::at_for(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT)),
            token,
            budget,
        )
    }
    #[cfg(test)]
    fn at(address: SocketAddr, token: &str) -> Result<Self, String> {
        Self::at_for(address, token, Duration::from_secs(12))
    }
    fn at_for(address: SocketAddr, token: &str, budget: Duration) -> Result<Self, String> {
        let deadline = Instant::now() + budget;
        if budget.is_zero() {
            return Err("หมดเวลารอ PhotoCraft".into());
        }
        if !address.ip().is_loopback()
            || token.len() != 64
            || !token.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("ข้อมูลเชื่อมต่อไม่ถูกต้อง กรุณาเปิด PhotoCraft ผ่าน Thai Font Kit".into());
        }
        let stream = TcpStream::connect_timeout(&address, budget.min(Duration::from_secs(3)))
            .map_err(|_| "ยังเชื่อมต่อไม่ได้ กรุณาเปิด PhotoCraft ผ่านปุ่มในแอป แล้วลองเชื่อมต่ออีกครั้ง")?;
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;
        let mut bridge = Self(BufReader::new(stream), deadline);
        bridge.call("auth", json!({"token":token}))?;
        Ok(bridge)
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let remaining = self
            .1
            .checked_duration_since(Instant::now())
            .ok_or("หมดเวลารอ PhotoCraft กรุณาตรวจงานก่อนลองใหม่")?;
        self.0
            .get_mut()
            .set_write_timeout(Some(remaining.min(Duration::from_secs(3))))
            .map_err(|e| e.to_string())?;
        let mut request = serde_json::to_vec(&json!({"id":1,"method":method,"params":params}))
            .map_err(|e| e.to_string())?;
        request.push(b'\n');
        self.0
            .get_mut()
            .write_all(&request)
            .map_err(|_| "การเชื่อมต่อขาด ไม่ส่งคำสั่งซ้ำ กรุณาตรวจงานใน PhotoCraft ก่อนลองใหม่")?;
        let mut line = Vec::new();
        loop {
            let remaining = self
                .1
                .checked_duration_since(Instant::now())
                .ok_or("หมดเวลารอ PhotoCraft งานอาจถูกใช้แล้ว กรุณาตรวจงานก่อนลองใหม่")?;
            self.0
                .get_mut()
                .set_read_timeout(Some(remaining))
                .map_err(|e| e.to_string())?;
            let buf = self
                .0
                .fill_buf()
                .map_err(|_| "PhotoCraft ตอบกลับไม่ทัน งานอาจถูกใช้แล้ว กรุณาตรวจงานก่อนลองใหม่")?;
            if buf.is_empty() {
                return Err("PhotoCraft ปิดการเชื่อมต่อ กรุณาตรวจงานก่อนลองใหม่".into());
            }
            let end = buf.iter().position(|b| *b == b'\n').map(|i| i + 1);
            let n = end.unwrap_or(buf.len());
            if line.len() + n > 1_048_576 {
                return Err("คำตอบ PhotoCraft ใหญ่เกินกำหนด".into());
            }
            line.extend_from_slice(&buf[..n]);
            self.0.consume(n);
            if end.is_some() {
                break;
            }
        }
        if line.len() > 1_048_576 || !line.ends_with(b"\n") {
            return Err("คำตอบจาก PhotoCraft ไม่สมบูรณ์".into());
        }
        let reply: Value =
            serde_json::from_slice(&line).map_err(|_| "รูปแบบคำตอบ PhotoCraft ไม่รองรับ")?;
        if reply["id"] != 1 {
            return Err("คำตอบ PhotoCraft ไม่ตรงกับคำขอ".into());
        }
        if reply["ok"] != true {
            // Authentication errors may echo input in future servers: never display them.
            if method == "auth" {
                return Err("ยืนยันการเชื่อมต่อไม่ได้ กรุณาเปิด PhotoCraft ผ่านแอปอีกครั้ง".into());
            }
            return Err(format!(
                "PhotoCraft: {}",
                reply["error"].as_str().unwrap_or("ไม่สามารถทำรายการได้")
            ));
        }
        Ok(reply["result"].clone())
    }
    fn execute(&mut self, command: &str, params: Value) -> Result<Value, String> {
        self.call("engine.execute", json!({"command":command,"params":params}))
    }
    pub fn check(&mut self) -> Result<(), String> {
        let fonts = self.execute("type.fonts", json!({}))?;
        if !fonts["families"].is_array() {
            return Err("PhotoCraft รุ่นนี้ยังไม่รองรับตัวเชื่อมฟอนต์".into());
        }
        Ok(())
    }
    pub fn apply(&mut self, family: &str) -> Result<(), String> {
        let fonts = self.execute("type.fonts", json!({"family":family}))?;
        if fonts["faces"].as_array().is_none_or(|f| f.is_empty()) {
            return Err(
                "PhotoCraft ยังไม่พบฟอนต์นี้ กรุณาติดตั้งฟอนต์ แล้วปิดและเปิด PhotoCraft ผ่านแอปอีกครั้ง".into(),
            );
        }
        let info = self
            .execute("type.info", json!({}))
            .map_err(|_| "กรุณาเปิดเอกสารและเลือกเลเยอร์ข้อความใน PhotoCraft ก่อนกดใช้ฟอนต์")?;
        let layer = info["layer"].as_u64().ok_or("ไม่พบเลเยอร์ข้อความที่เลือก")?;
        // One explicit edit, never retry or save the user's document automatically.
        self.execute("type.setStyle", json!({"layer":layer,"font":family}))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    #[test]
    fn authenticated_apply_targets_text_layer_once() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut r = BufReader::new(stream);
            for (command, result) in [
                ("auth", json!({})),
                ("type.fonts", json!({"faces":[{}]})),
                ("type.info", json!({"layer":7})),
                ("type.setStyle", json!({})),
            ] {
                let mut line = String::new();
                r.read_line(&mut line).unwrap();
                let v: Value = serde_json::from_str(&line).unwrap();
                if command == "auth" {
                    assert_eq!(v["method"], "auth");
                } else {
                    assert_eq!(v["params"]["command"], command);
                }
                if command == "type.setStyle" {
                    assert_eq!(v["params"]["params"], json!({"layer":7,"font":"Sarabun"}));
                }
                writeln!(r.get_mut(), "{}", json!({"id":1,"ok":true,"result":result})).unwrap();
            }
        });
        Bridge::at(address, &"a".repeat(64))
            .unwrap()
            .apply("Sarabun")
            .unwrap();
        server.join().unwrap();
    }
    #[test]
    fn slow_partial_reply_cannot_extend_total_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            for _ in 0..10 {
                if stream.write_all(b" ").is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
        });
        let start = Instant::now();
        assert!(Bridge::at_for(address, &"a".repeat(64), Duration::from_millis(120)).is_err());
        assert!(start.elapsed() < Duration::from_millis(350));
        server.join().unwrap();
    }
    #[test]
    fn rejects_remote_address_and_invalid_token() {
        assert!(Bridge::at("192.0.2.1:17878".parse().unwrap(), &"a".repeat(64)).is_err());
        assert!(Bridge::connect("short").is_err());
    }
    #[test]
    fn failed_authentication_never_leaks_token() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut r = BufReader::new(stream);
            let mut line = String::new();
            r.read_line(&mut line).unwrap();
            writeln!(
                r.get_mut(),
                "{}",
                json!({"id":1,"ok":false,"error":"a".repeat(64)})
            )
            .unwrap();
        });
        let error = Bridge::at(address, &"a".repeat(64)).err().unwrap();
        assert!(!error.contains(&"a".repeat(64)));
        server.join().unwrap();
    }
}
