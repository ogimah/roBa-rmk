Vial 0.7.5 BLE通信試験版（非公式・診断専用）

1. ZIPを新しいフォルダーへ展開してください。既存Vialには上書きしません。
2. 通常Vial・Web Vial・Pipetteを閉じ、キーボードをBluetooth接続します。
3. Vial BLE Probe.exe を起動し、右上の Refresh を1回押します。
4. アプリを閉じ、EXEと同じフォルダーの vial-ble-diagnostic.log を送ってください。

キーボード設定の編集、.vilの読込み、アンロックは不要です。
ファームの書込み、ペアリング削除、レジストリ変更は行いません。

本試験版は、次の条件をすべて満たす機器だけを一時的にVial候補として通します。
  VID 4C4B / PID 4643、Usage Page FF60 / Usage 0061、HIDシリアルが空
実機・Windowsの識別情報は変更せず、列挙結果のメモリー上のコピーにだけ診断用シリアルを付けます。

ログは対象ファームとVial用Raw HIDの識別情報、通信開始の成功・失敗を記録します。
機器名、機器のシリアル、Bluetooth機器パスが含まれます。
キーマップ・TD・マクロの内容やWindowsのアカウント名は採取しません。
EXEフォルダーに書込みできない場合のログ保存先:
  %LOCALAPPDATA%\Vial BLE Probe\vial.log

公式Vial 0.7.5ソース:
https://github.com/vial-kb/vial-gui/tree/e42622c352b69eff523a467718db487ab9c9cbf2
Vial本体はGPL-2.0-or-laterです。COPYINGと対応する診断用ソースを同梱しています。
HIDライブラリは公式Vial 0.7.5と同一です。上記の限定候補以外の検出条件は変更しません。
