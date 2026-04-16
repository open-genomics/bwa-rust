## 📝 Description

Briefly describe the changes in this PR.

## 🏷️ Type of Change

- [ ] 🐛 Bug fix (non-breaking change that fixes an issue)
- [ ] ✨ New feature (non-breaking change that adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to change)
- [ ] ⚡ Performance improvement
- [ ] 🔧 Refactoring (no functional changes)
- [ ] 📚 Documentation update
- [ ] 🧪 Test improvement
- [ ] 🔨 CI/CD related

## 🔗 Related Issue

Closes #

## 📋 Changes Made

List the key changes:
- Change 1
- Change 2
- Change 3

## ✅ Checklist

### Code Quality
- [ ] Code passes `cargo fmt --all -- --check`
- [ ] Code passes `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] No `unsafe` code added (project forbids unsafe)

### Testing
- [ ] All tests pass: `cargo test --all-targets --all-features`
- [ ] New tests added for new functionality
- [ ] Edge cases tested

### Documentation
- [ ] Documentation updated if needed
- [ ] Doc comments added for new public APIs
- [ ] CHANGELOG.md updated (if applicable)

### Compatibility
- [ ] No breaking changes (or clearly documented)
- [ ] Index format compatibility maintained (or version bumped)

## 🧪 Testing

Describe how to verify these changes:

```bash
# Commands to test the changes
cargo test <test_name>
cargo run --example simple_align
```

## 📊 Performance Impact (if applicable)

| Metric | Before | After |
|--------|--------|-------|
| Time | | |
| Memory | | |

## 📸 Screenshots (if applicable)

Add screenshots here if relevant.

---

**Reviewer Guidelines:**
- Check that all CI checks pass
- Verify the changes align with project architecture
- Test the changes locally if needed
