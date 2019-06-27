# msh

## 実行

```
$ cargo run
```

## 仕様

- 変数名として使える文字は英数字と'-'と'\_'
- コマンドの優先順位
    - 組み込み関数
    - $PATH で見つかった順
- シェルの実行中にエラーが起きた場合、終了コード1で終了する
- リダイレクト出力でファイルが作られるときのパーミッションは`-rw-r--r--`(にumaskを掛けたもの)
- エイリアスでは(現状では)単純な「コマンド+引数」の置換しか出来ない
- エイリアスの文法は"alias name = content"で、bashなどのシェルとは異なる
- シェル変数の定義は"var name = content"で、bashなどのシェルとは異なる
- 環境変数の定義は"export name = content"で、bashなどのシェルとは異なる

- 実装をサボってunwrapしてる箇所が結構あるのでpanicすることがありそう

### 文法

|表記|機能|
|----|----|
|`;`|コマンドの連結|
|<code>&#124;</code>|パイプ(標準出力)|
|<code>&#124;&amp;</code>|パイプ(標準エラーも含めて渡す)|
|`>`|ファイルを作って書き込む|
|`>=`|ファイルに書き出す(上書き)|
|`>+`|追記|
|`>!`|ファイルを作って書き込む(標準エラー出力)|
|`>=!`|ファイルに書き出す(上書き)(標準エラー出力)|
|`>+!`|追記(標準エラー出力)|
|`<`|ファイルから読み込む|
|`<<`|ヒアドキュメント|
|`()`|サブシェルを作成|
|`$()`|サブシェルを作成して実行した結果の文字列|
|`${変数名}`|変数を文字列に展開|
|`""`|空白文字をまとめる|

#### 条件実行

- `&&`でコマンドを繋ぐと、前のコマンドの終了コードが0の場合のみ次のコマンドが実行される。
- `||`でコマンドを繋ぐと、前のコマンドの終了コードが0以外の場合のみ次のコマンドが実行される。

### 組み込み関数

|関数|説明|
|----|----|
|`alias`|エイリアスを設定する|
|`unalias`|エイリアスを削除する|
|`cd`|カレントディレクトリを変更する|
|`type`|コマンド名の実体を調べる|
|`exit`|シェルを終了する|
|`export`|環境変数を設定する|
|`var`|シェル変数を設定する|
|`unset`|シェル変数・環境変数の削除|
|`reload-path`|$PATHの再検索|
