# テスト (tests)

`pytest` を使用した自動テストコードが含まれています。

## 実行方法
```bash
uv run pytest
```

## 構成
- `conftest.py`: テスト共通設定・モック定義
- `infrastructure/`: インフラ層の単体テスト
- `services/`: サービス層の単体テスト
