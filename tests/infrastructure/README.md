# インフラ層テスト (tests/infrastructure)

`src/infrastructure/` 配下のコンポーネントに対する単体テストを配置しています。I/O をモックし、外部デバイスやネットワークに依存しない形で動作確認します。

## 主なテスト
- `test_audio_recorder.py`: 録音開始・停止のスレッド管理とファイル生成の振る舞いを検証
- `test_transcriber.py`: Whisper ベースの文字起こしが遅延ロードで単一インスタンスを使うことを確認
