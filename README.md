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


## 動作例

### 単一コマンド
```
[algon@ThinkPad-X1-Carbon:~/msh (master %)]$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/msh`
msh version 0.2.0
algon@/home/algon/msh $ ls
Cargo.lock  Cargo.toml	README.md  build.rs  src  target  test
=> 0
```
### 標準入出力の切り替え
```
algon@/home/algon/msh $ wc < Cargo.toml > wc_res
=> 0
algon@/home/algon/msh $ cat wc_res
 11  29 199
=> 0
algon@/home/algon/msh $ wc < Cargo.toml > wc_res
error: `wc_res` already exists, use `>=` to overwrite it.
algon@/home/algon/msh $ wc < Cargo.toml >= wc_res
=> 0
```
### パイプライン
```
algon@/home/algon/msh $ cd test	
=> 0
algon@/home/algon/msh/test $ vim data1
=> 0
algon@/home/algon/msh/test $ vim data2
=> 0
algon@/home/algon/msh/test $ cat data1 data2
a
aa
aa
aaa
aaa
bbb
bbb
b
aaa
=> 0
algon@/home/algon/msh/test $ cat data1 data2 | sort | uniq
a
aa
aaa
b
bbb
=> 0
```
### 終了
```
algon@/home/algon/msh/test $ exit
good bye.
[algon@ThinkPad-X1-Carbon:~/msh (master %)]$ 
```
### 標準エラー出力の切り替え
```
algon@/home/algon/msh/test $ vim ce.c
=> 0
algon@/home/algon/msh/test $ cat ce.c
#include <stdio.h>
int main() {
    printf("compile error")
    return 0;
}
=> 0
algon@/home/algon/msh/test $ gcc ce.c  
ce.c: In function ‘main’:
ce.c:4:5: error: expected ‘;’ before ‘return’
     return 0;
     ^~~~~~
=> 1
algon@/home/algon/msh/test $ gcc ce.c |& tee error
ce.c: In function ‘main’:
ce.c:4:5: error: expected ‘;’ before ‘return’
     return 0;
     ^~~~~~
=> 0
algon@/home/algon/msh/test $ cat error
ce.c: In function ‘main’:
ce.c:4:5: error: expected ‘;’ before ‘return’
     return 0;
     ^~~~~~
=> 0
algon@/home/algon/msh/test $ vim test.c
=> 0
algon@/home/algon/msh/test $ gcc test.c
=> 0
algon@/home/algon/msh/test $ ./a.out	
stdout
stderr
=> 0
algon@/home/algon/msh/test $ ./a.out > out >! err
=> 0
algon@/home/algon/msh/test $ cat out
stdout
=> 0
algon@/home/algon/msh/test $ cat err
stderr
=> 0
```
### サブシェルの実行
```
algon@/home/algon/msh/test $ (cat | sort | uniq >= result) < data1
=> 0
algon@/home/algon/msh/test $ cat result
a
aa
aaa
=> 0
```
### バックグラウンド実行
ジョブ管理機能は実装出来ていないため、簡易的なもの(フォアグラウンドで実行したりすることは出来ない)
```
algon@/home/algon/msh/test $ sleep 3; echo finished &
[run on the background]
=> 0
algon@/home/algon/msh/test $ finished

```
### 条件実行
```
algon@/home/algon/msh/test $ gcc ce.c && echo "ok"
ce.c: In function ‘main’:
ce.c:4:5: error: expected ‘;’ before ‘return’
     return 0;
     ^~~~~~
=> 1
algon@/home/algon/msh/test $ gcc test.c && echo "ok"
ok
=> 0
algon@/home/algon/msh/test $ gcc ce.c || echo "error"
ce.c: In function ‘main’:
ce.c:4:5: error: expected ‘;’ before ‘return’
     return 0;
     ^~~~~~
error
=> 0
```
### シェル組み込みコマンド
#### cd
引数に`-`を指定すると直前にいたディレクトリに移動する。引数を省略すると`$HOME`の指すディレクトリに移動する。
```
algon@/home/algon/msh/test $ cd ..
=> 0
algon@/home/algon/msh $ cd -
=> 0
algon@/home/algon/msh/test $ cd
=> 0
algon@/home/algon $ 
```
#### alias / unalias
「コマンド名 + 引数」をまとめるだけの簡易的なもの
```
algon@/home/algon/msh/test $ alias hello = "echo hello"
=> 0
algon@/home/algon/msh/test $ hello 
hello
=> 0
algon@/home/algon/msh/test $ unalias hello
=> 0
algon@/home/algon/msh/test $ hello
error: `hello` not found.
```
#### type
引数に指定したコマンドが外部コマンドなのか組み込みコマンドなのか、エイリアスなのかを表示する。
```
algon@/home/algon/msh/test $ type cd
`cd` is a builtin function.
=> 0
algon@/home/algon/msh/test $ type ls
/bin/ls
=> 0
algon@/home/algon/msh/test $ type hello
`hello` not found.
=> 0
algon@/home/algon/msh/test $ alias hello = "echo hello"
=> 0
algon@/home/algon/msh/test $ type hello
`hello` is an alias of `echo hello`
=> 0
```
#### exit
シェルを終了する。
```
algon@/home/algon/msh/test $ exit
good bye.
```

#### export / var / unset
```
algon@/home/algon/msh/test $ export HOGE = "hoge"
=> 0
algon@/home/algon/msh/test $ sh 
$ echo $HOGE
hoge
$ 
=> 0
algon@/home/algon/msh/test $ var xxx = 123  
=> 0
algon@/home/algon/msh/test $ echo $xxx
123
=> 0
algon@/home/algon/msh/test $ unset xxx
=> 0
algon@/home/algon/msh/test $ echo $xxx

=> 0
```
#### reload-path
`$PATH`内のコマンドを再検索する。
### 変数展開
```
algon@/home/algon/msh/test $ var msg = "hello"
=> 0
algon@/home/algon/msh/test $ echo $msg
hello
=> 0
algon@/home/algon/msh/test $ echo "${msg} world"
hello world
=> 0
```
### コマンド置換
```
algon@/home/algon/msh/test $ echo run on $(uname)
run on Linux
=> 0
algon@/home/algon/msh/test $ echo nest $(echo is also $(echo supported))
nest is also supported
=> 0
```