# 次のステップ: JSON-LD IR完全移行

## Phase 4: Rust型の完全削除とJSON-LD直接操作API

### 1. IR型の完全置き換え
- [ ] `RuleIR`, `QueryIR`, `PatchIR`, `StrategyIR`のRust型を削除
- [ ] JSON-LD `Value`を直接操作するAPIに変更
- [ ] `kotoba-ir`クレートをJSON-LD専用APIに再設計

### 2. 使用箇所の更新
- [ ] `crates/010-logic/012-kotoba-rewrite-kernel/src/rule.rs`をJSON-LD IR対応に更新
- [ ] `crates/020-language/028-kotoba2tsx/src/render_ir.rs`をJSON-LD IR対応に更新
- [ ] その他のIR使用箇所をすべて更新

### 3. Catalog-IRのJSON-LD化
- [ ] Catalog-IRのOWLオントロジー定義を追加
- [ ] Catalog-IRのSHACL Shape定義を追加
- [ ] Catalog-IRのJSON-LD変換機能を実装
- [ ] Rust型の削除

### 4. WASMランタイム統合
- [ ] WASMランタイムインターフェースの設計
- [ ] JSON-LD IRをWASMで実行する仕組みの実装
- [ ] fukurow WASMエンジンとの統合

### 5. SHACL検証の必須化
- [ ] すべてのIR操作でSHACL検証を必須に
- [ ] 検証失敗時のエラーハンドリング強化

### 6. パフォーマンス最適化
- [ ] JSON-LD IRのパース/シリアライズ最適化
- [ ] CIDベースのキャッシングシステム
- [ ] メモ化による重複処理の削減

## 実装順序

1. **Catalog-IRのJSON-LD化** (既存システムへの影響が小さい)
2. **IR型の完全置き換え** (中核機能)
3. **使用箇所の更新** (影響範囲が広い)
4. **WASMランタイム統合** (新機能)
5. **SHACL検証の必須化** (品質向上)
6. **パフォーマンス最適化** (最終調整)

