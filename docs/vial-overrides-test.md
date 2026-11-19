# COROPIT版：Vial Key Override 試験

対象ブランチ：`test/vial-overrides-coropit`。基準は既存COROPIT版
`8351a405eaf7e47c9788bfe832943d4f445d7441`（RMK 0.9.0）。
main・配布リポジトリは対象外。純正版と共通のOverride・ホイール対策を維持する。

## 試し方

1. **今の実機設定をVialから `.vil` に保存**する。以前のファイルで代用せず、GT1〜3、ST、仮想十字、TD、マクロ、コンボを含む現状を保存する。
2. このブランチの成功した [Actions](https://github.com/ogimah/roBa-rmk/actions/workflows/build.yml?query=branch%3Atest%2Fvial-overrides-coropit) を開き、下部のArtifactsから次を取得する（GitHubログインが必要）。
   - 右／central：`roBa_RMK_GT_COROPIT_OverrideTest-central.uf2`
   - 左／peripheral：`roBa_RMK_GT_COROPIT_OverrideTest-peripheral.uf2`
3. COROPITを使用するキーボードの左右に対応するファイルを書き込む。設定が初期化された場合は、手順1の `.vil` を読み戻す。
4. **まずVialの `Key Overrides` タブ**を使う。32枠が表示される。初期値は全枠空。
5. 既存の「Ctrl＋テンキー1 → Ctrl＋通常1」コンボだけ一時的に無効にする。同じ入力をコンボとOverrideの両方に割り当てると、どちらが働いたか確認できない。
6. Overrideを1件設定する。

| 項目 | 初回試験の設定 |
|---|---|
| Enable | ON |
| Enable on layers | 全レイヤ |
| Trigger | テンキー1（`KC_P1`） |
| Trigger mods | 左Ctrl・右Ctrlの両方 |
| Negative mods | なし |
| Suppressed mods | **なし**（Ctrlを残す） |
| Replacement | 通常の1（`KC_1`）。ここにさらにCtrlを付けない |
| Options | 最初の3項目（トリガ押下／必要な修飾キー押下／除外修飾キー解放で有効化）をON。他はOFF |

左右Ctrlの両方を指定すると、左右どちらかで成立する。初回の設定は有効ビット込みで options `0x87`。
現在確認したVial GUIソースでは最後の2つのOptionsの表示文と内部名に食い違いがあるため、初回試験では両方OFFにする。

## 実機で確認すること

- テンキー1単独はテンキー1のまま。
- 左Ctrl＋テンキー1、右Ctrl＋テンキー1で **Ctrl＋通常1** が届く。
- 数字を先に押してからCtrlを押しても切り替わる。Ctrlを先に離してから数字を離してもキーが残らない。
- 数字レイヤ用LT＋Ctrl＋テンキー1でも動く。これはコンボ同時押し数の枠を使わない。
- USBを抜き差しした後もOverrideが残る。`.vil` の保存・読み戻しでも残る。
- 通常入力、TD、マクロ、GT1〜3、ST、エンコーダー、左右の接続が従来どおり動く。

問題なければ同じ設定でテンキー0〜9に拡張する。コンボとの重複は各数字について解消する。
戻す場合は保存済みの正常版UF2を書き込み、手順1の `.vil` を読み戻す。

## 実装と検証

Ergohaven [75186785](https://github.com/ergohaven/rmk/commit/751867859534445c22fd138ea07d9ce72712f4d3)
のVial Key Override対応を、現在の0.9.0へ移植。Ergohaven全体への依存切替は行わず、
元のライセンス・著作者情報を維持した。必要な3クレートを同一の安定版からvendor化した。
第三者ライブラリのバージョンは3つのlockfileとも変更していない。

- Vial通信の32枠通知・10バイトGET/SET、即時反映、無効化、レイヤ条件、左右修飾キー、除外・抑制を接続。
- 既存のフラッシュ処理を再利用。実際のVial SETからストレージタスクを動かし、フラッシュ内容を保持してStorage/KeyMapを作り直し、Vial GETで復元を確認する試験を追加。
- 実際のKeyboard処理からUSB HIDキューに出るレポートまで確認。テンキー0〜9、左右Ctrl、Ctrl＋Shift、押下／解放順、複数数字、LTとの併用、範囲外アクセス、最終スロットを試験。
- 条件不成立時にも修飾キーを抑制する問題と、抑制解除後にホストへ修飾キー復帰を送らない問題を、レポート試験で検出して修正。
- センサー設定、キーマップ・TD・マクロ、GT/ST、エンコーダーの修正部分は維持。`keyboard.toml` は `fork_max_num = 32` の追加だけ。

設定保持17件、既存ホスト試験19件、ライブラリ247件がローカルで成功。
247件のうち自動マウスレイヤの単体試験は2枠を仮定するため、全体試験時だけ
テスト用設定で容量を2にする。製品設定の容量0では改修前にも同じ11件が失敗することを比較確認した。
Override・Fork・エンコーダーの18件は**製品設定そのもの**でも成功。
この設定補助は製品の自動マウスレイヤを有効にする変更ではない。

ビルドとソフトウェア試験の成功は、実機／GUIでの動作確認とは別。
上記の実機確認が終わるまでは試験版として扱う。
