use std::collections::HashMap;
use std::fs;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_yaml::Value;
use rcon::Connection;
use tokio::time::{timeout, Duration};

// --- 設定読み込み ---
fn load_config(path: &str) -> (u16, HashMap<String, (String, String)>, String) {
    let s = fs::read_to_string(path).expect("Cannot read YAML file");
    let yaml: Value = serde_yaml::from_str(&s).expect("Invalid YAML");

    let control_port = yaml.get("control_port")
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .unwrap_or(3577);

    let common_password = yaml.get("common_password")
        .and_then(|v| v.as_str())
        .unwrap_or("kr_mc")
        .to_string();

    let mut rcon_map = HashMap::new();

    if let Some(servers) = yaml.get("servers") {
        if let Some(obj) = servers.as_mapping() {
            for (k, v) in obj {
                let key = k.as_str().unwrap().to_string();
                if key.starts_with("rcon") {
                    if let Some(mapping) = v.as_mapping() {
                        let addr = mapping.get(&Value::from("addr")).unwrap().as_str().unwrap().to_string();
                        let password = mapping.get(&Value::from("password")).unwrap().as_str().unwrap().to_string();
                        rcon_map.insert(key, (addr, password));
                    }
                }
            }
        }
    }

    (control_port, rcon_map, common_password)
}

// --- RCON実行 ---
async fn send_rcon(target: &str, command: &str, rcon_map: &HashMap<String, (String, String)>) -> String {
    if let Some((addr, password)) = rcon_map.get(target) {
        match timeout(Duration::from_secs(10), async {
            let mut conn = Connection::builder()
                .connect(addr, password)
                .await?;
            conn.cmd(command).await
        }).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => format!("RCON command failed: {}", e),
            Err(_) => "RCON connect timeout".to_string(),
        }
    } else {
        format!("RCON target {} not found", target)
    }
}


// --- TCP受信リスナー ---
async fn start_tcp_listener(control_port: u16, rcon_map: HashMap<String, (String, String)>, common_password: String) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", control_port)).await.unwrap();
    println!("Listening on TCP {}", control_port);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let rcon_map = rcon_map.clone();
        let common_password = common_password.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            let n = match timeout(Duration::from_secs(10), socket.read(&mut buf)).await {
                Ok(Ok(n)) => n,
                _ => {
                    let _ = socket.write_all(b"Read timeout\n").await;
                    return;
                }
            };

            let msg = String::from_utf8_lossy(&buf[..n]);
            for line in msg.lines() {
                if let Some((target, rest)) = line.split_once(":!") {
                    let (command, password) = if let Some((cmd, pw)) = rest.split_once(":\\") {
                        (cmd.trim_matches(|c| c == '{' || c == '}'), pw)
                    } else {
                        (rest.trim_matches(|c| c == '{' || c == '}'), "")
                    };

                    if password != common_password {
                        let _ = socket.write_all(b"Invalid password\n").await;
                        continue;
                    }

                    let result = send_rcon(target, command, &rcon_map).await;
                    let _ = socket.write_all(result.as_bytes()).await;
                }
            }
        });
    }
}

// --- メイン ---
#[tokio::main]
async fn main() {
    let (control_port, rcon_map, common_password) = load_config("rcsh.yml");

    start_tcp_listener(control_port, rcon_map, common_password).await;
}
