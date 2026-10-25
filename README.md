# JastProgram
JastProgramは、HaxeやJavaScript、Rust、C++等の構文を取り入れた、汎用プログラミング言語です。

## 概要
JastProgramの基本的な構文はHaxeやJavaScriptに寄せています。

## Hello, world!
`main.jp`
```jp
function main(): Void {
    print("Hello, world!");
}
```

## 実行 & コンパイル & トランスパイル
#### 実行
```pwsh
jacm -main main.jp --interp
```

#### コンパイル後に実行
```pwsh
nbs main.jp -o main
```
こうすると、`main`と名付けられたコンパイル済みファイルが作成されます。
その後
```pwsh
/helloWorld
```
を使用して実行します。