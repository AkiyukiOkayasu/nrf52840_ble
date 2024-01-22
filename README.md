# nrf52840_ble

nRF52840とEmbassyを使うBLEのサンプル。  
プロトコルスタックにはSoftDeviceを用います。

## 必要なハードウェア

- 秋月電子通商 / [nRF52840使用BLEマイコンボード](https://akizukidenshi.com/catalog/g/gK-17484/)
- J-Link

nRF52840使用BLEマイコンボードはSWDのピンヘッダーにハンダ付けしてJ-Linkと接続します。


## 環境構築

Rustはすでにインストールされているものとします。　　

最新のRustにアップデート。  
`rustup update`

thumbv7em-none-eabihfのターゲットを追加。  
nRF52840はCortex-M4Fなのでクロスコンパイルをするためにthumbv7em-none-eabihfのツールチェインを追加します。  
`rustup target add thumbv7em-none-eabihf`

probe-rsをインストール。  
`cargo install probe-rs --features cli`

## SoftDeviceの書き込み

SoftDevice S140 v7.3.0を[ここ](https://www.nordicsemi.com/Products/Development-software/s140/download)からダウンロードします。  

J-LinkとnRF52840使用BLEマイコンボードを接続しFlashの消去した後にSoftDeviceを書き込みます。これは最初の1度のみ必要です。　　

Flashの消去。  
`probe-rs erase --chip nrf52840`

SoftDeviceの書き込み  
`probe-rs download --chip nrf52840 --format hex s140_nrf52_7.3.0_softdevice.hex`


## Run

以下のコマンドで実行します。  
`cargo run --release`

[LightBlue](https://punchthrough.com/lightblue/)などのアプリを用いて、BLE接続します。  
デバイス名はHelloRustです。  

BatteryLevelなどの表示されていることを確認します。  

## 参考

### SoftDeviceとメモリマップ

SoftDeviceはFlashの先頭に書き込まれ、Rustプログラムはその後ろに書き込まれます。そのためのSoftDeviceが使用するFlashの量を確認し、memory.xのFlashの先頭をその分オフセットします。  
ダウンロードしたSoftDeviceのリリースノートを確認してFlashの量を確認します。  
SoftDevice S140 v7.3.0（s140_nrf52_7.3.0_release-notes.pdf）では、　　
> • Flash: 156.0 kB (0x27000 bytes).
と記載がありますので、memory.xのFlash位置をその分オフセットし、LENGTHのその分縮めています。
https://github.com/AkiyukiOkayasu/nrf52840_ble/blob/8d3d7c34e4d218fab91a25ef6be197e3c2793fbb/memory.x#L4-L8

RAMの使用量は起動時にprintされるので、それに従って設定します。

### Example

Embassy公式のsoftdeviceの[Example](https://github.com/embassy-rs/nrf-softdevice/blob/master/examples/src/bin/ble_bas_peripheral.rs)もあります。このリポジトリはそれを整理し、簡素化したものです。




