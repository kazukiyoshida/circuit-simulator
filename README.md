![](https://user-images.githubusercontent.com/11558164/72995577-80516800-3e3c-11ea-94ae-f02f595db0f2.png)

# Circuit Simulator

Rust で記述する電子回路シミュレータです.  
WebAssembly 対応しており、npm モジュールとして使用できます.

## Description

スクラッチで開発した電子回路シミュレータであり、以下のような特徴を持ちます.
- 直流解析 （.DC) にのみ対応しています.
- 修正節点法（Modified Nodal Analysis）を実装しています
    - SPICE-like な回路シミュレータとなっています
- 非線形素子をデバイスモデルとして保持しており、Newton法を用いて方程式を解いています.
- WebAssembly インターフェイスを備えています.

## Install & Setup

#### サンプルコードの実行

```sh
$ cargo run --bin core
```

## Publish

#### 1. パッケージのビルド

Rust コードを WebAssembly にコンパイルし、npm モジュールを作成します.

```
wasm-pack build --scope kazukiyoshida
```

#### 2. Verdaccio の起動

回路シミュレータはプライベートレジストリに公開するので、verdaccio を立ち上げておきます。

```sh
$ verdaccio
```
これにより http://localhost:4873/ が立ち上がります。

#### 3. verdaccio プライベートレジストリにパッケージを公開

```sh
npm publish --registry http://localhost:4873
```
