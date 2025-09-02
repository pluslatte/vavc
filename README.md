# vavc
## 概要
アバター変更を行うためのコマンドラインツールです
## ビルド
- `cargo build --release`
- 適当にインストールしてください
## 使い方
vavc という名前で呼び出せるようにしたなら
### 基本
- `vavc auth new`
- `vavc fetch`
- `vavc switch -i <アバターのid>`
- `vavc switch -q <アバター名称>`
### おまけ
- `vavc alias set -a <好きな別名> -i <アバターのid>` or `vavc alias set -a <好きな別名> -q <アバター名称>`
- `vavc switch -a <登録した別名>`
- `vavc search`
- `vavc list`
- `vavc auth check`