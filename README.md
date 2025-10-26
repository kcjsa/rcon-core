rcsh_control_core

rcsh_control_core は、TCP 経由で RCON コマンドを Minecraft サーバーに送信できる Rust 製のコントロールコアです。
単一のパスワード（Core マスターパスワード）で複数サーバーを一元管理し、セキュアにコマンドを送信できます。

特徴

TCP ポート（デフォルト: 3577）で Core に接続

Core 経由で複数の RCON サーバーを操作可能

サーバーごとに異なる RCON パスワードをサポート

非同期通信 (Tokio) に対応

YAML 形式で簡単に設定可能

SSH は省略（RCON 専用）

動作環境

Rust 1.72 以降推奨

Linux / Windows / macOS

Minecraft サーバー（Java Edition）で RCON 有効化済み

インストール
git clone <リポジトリURL>
cd rust_core
cargo build --release


ビルド後、実行ファイルは target/release/rcsh_control_core に生成されます。

設定ファイル

ファイル名: rcsh.yml

control_port: 3577  # Core が待ち受けるTCPポート
common_password: "kr_mc"  # Core 経由用の共通パスワード

servers:
  rcon1:
    addr: "192.168.1.100:25575"
    password: "kr-mc"
  rcon2:
    addr: "192.168.1.101:25575"
    password: "bacon"


addr は IP:Port 形式

password は各サーバーの RCON パスワード

使用方法

TCP クライアントから Core に接続してコマンド送信:

echo 'rcon1:!say Hello World:\kr_mc' | nc 127.0.0.1 3577


書式: ターゲット:!コマンド:\Coreパスワード

例: rcon1:!say Hello:\kr_mc
