
#### 方案 B：迁移到 ethers 生态（推荐长期）
切换到 ethers 生态常用的 `coins-bip39` + `coins-bip32`：
```toml
[dependencies]
ethers = { version = "2.0", features = ["mnemonic"] }
coins-bip39 = "0.8"
coins-bip32 = "0.8"
# 删除: bip39, bip32, k256, sha2
```
用户确认按照方案 B 进行修改。我需要：
1. 修改 Cargo.toml - 替换 bip39/bip32/k256/sha2 为 coins-bip39/coins-bip32
2. 修改 wallet/mod.rs - 适配新的 API
3. 确保所有测试能通过

&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&7
## ✅ 修改完成

### 📦 Cargo.toml 变更

```toml
# 删除的依赖
- bip39 = { version = "2.0", features = ["rand", "zeroize"] }
- bip32 = { version = "0.5", features = ["bip39", "secp256k1"] }
- k256 = { version = "0.13", features = ["ecdsa", "sha256"] }
- sha2 = "0.10"
- ethers-core = "2.0"  # dev-dependencies

# 新增的依赖
+ coins-bip39 = "0.8"
+ coins-bip32 = "0.8"
```

### 📝 wallet/mod.rs 关键变更

1. **导入更新**：
   ```rust
   use coins_bip32::prelude::*;
   use coins_bip39::{English, Mnemonic as CoinsMnemonic};
   // 使用 ethers::core::k256 替代独立的 k256
   ```

2. **助记词验证与生成**（`coins-bip39` API 差异）：
   ```rust
   // 验证
   CoinsMnemonic::<English>::new_from_phrase(phrase)
   
   // 生成（需要转换 word count -> entropy bits）
   CoinsMnemonic::<English>::new_with_count(&mut rand::thread_rng(), entropy_size)
   ```

3. **密钥派生**（`coins-bip32` API 差异）：
   ```rust
   let derivation_path: DerivationPath = path.parse()?;
   let derived_key = XPrv::derive_from_path(&self.seed, &derivation_path)?;
   ```

4. **加密/解密**：使用 `ethers::utils::keccak256` 替代 `sha2`

5. **seed 存储**：从 `bip32::Seed` 改为原始 `[u8; 64]`

### 🗑️ 清理文件
- 删除了未使用的 `wallet/hd_wallet.rs`（旧实现）

---

### ⚠️ 注意事项

由于环境缺少 Visual Studio 编译工具链，未能运行 `cargo test` 验证。建议在支持的环境中执行：

```bash
cd crates/chain
cargo test wallet:: --lib
```

`coins-bip39` 与 `coins-bip32` 是 ethers 生态常用 crate，API 与原来的 `bip39`/`bip32` 有差异，主要变化：
- `coins-bip39` 使用泛型 `Mnemonic<English>` 而不是语言参数
- 生成助记词需要指定 entropy bits（128/160/192/224/256）而非 word count
- `coins-bip32` 的 `XPrv::derive_from_path` 直接返回私钥 bytes


