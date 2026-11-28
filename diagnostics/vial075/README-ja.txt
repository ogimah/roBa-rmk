Vial 0.7.5 BLE診断版（非公式・診断専用）

1. ZIPを新しいフォルダーへ展開してください。既存Vialには上書きしません。
2. 通常Vial・Web Vial・Pipetteを閉じ、キーボードをBluetooth接続します。
3. Vial BLE Diagnostic.exe を起動し、右上の Refresh を1回押します。
4. アプリを閉じ、EXEと同じフォルダーの vial-ble-diagnostic.log を送ってください。

キーボード設定の編集、.vilの読込み、アンロックは不要です。
ファームの書込み、ペアリング削除、レジストリ変更は行いません。

ログは対象ファーム（VID 4C4B / PID 4643）とVial用Raw HIDの識別情報を記録します。
機器名、機器のシリアル、Bluetooth機器パスが含まれます。
キーマップ・TD・マクロの内容やWindowsのアカウント名は採取しません。
EXEフォルダーに書込みできない場合のログ保存先:
  %LOCALAPPDATA%\Vial BLE Diagnostic\vial.log

公式Vial 0.7.5ソース:
https://github.com/vial-kb/vial-gui/tree/e42622c352b69eff523a467718db487ab9c9cbf2
Vial本体はGPL-2.0-or-laterです。COPYINGと対応する診断用ソースを同梱しています。
本診断版は、検出条件やHIDライブラリを変更せず、ログ採取だけを追加します。
