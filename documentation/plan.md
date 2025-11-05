# VectDB Development Plan

**Last Updated**: 2025-11-05
**Current Phase**: 7 (Polish & Documentation)
**Target Release**: v0.1.0

## Overview

This document outlines the roadmap for VectDB development, organized by priority and timeframe. All work follows the Test-Driven Development (TDD) process defined in [process.md](./process.md).

## Immediate Priorities (This Week)

### 1. Complete Phase 7 Documentation ✅

**Status**: In Progress

**Tasks**:
- [x] Create comprehensive documentation structure
- [x] Write architecture.md (hexagonal architecture, data flows)
- [x] Write prd.md (product requirements, user stories)
- [x] Write design.md (component designs, algorithms)
- [x] Write process.md (TDD workflow, quality checks)
- [x] Write status.md (current state, metrics)
- [x] Write plan.md (this document)
- [ ] Update README.md with links to documentation
- [ ] Update CLAUDE.md with process reminders

**Acceptance Criteria**:
- All documentation files created and reviewed
- README references all documentation
- CLAUDE.md includes process checklist
- All docs pass markdown linting

**Estimated Effort**: 1 day

### 2. Quality Audit

**Status**: Not Started

**Tasks**:
- [ ] Run full quality check suite:
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- [ ] Verify .gitignore covers all cases
- [ ] Check for any TODOs or FIXMEs in code
- [ ] Review all public API documentation
- [ ] Test all CLI commands manually
- [ ] Test web UI with MCP/Playwright

**Acceptance Criteria**:
- Zero clippy warnings
- All tests passing
- All public APIs documented
- .gitignore verified
- Manual testing complete

**Estimated Effort**: 4 hours

### 3. Example Dataset Creation

**Status**: Not Started

**Tasks**:
- [ ] Create `examples/datasets/rust-docs/` with curated Rust documentation excerpts
- [ ] Create `examples/datasets/ml-papers/` with public domain ML papers
- [ ] Create `examples/datasets/code-snippets/` with code examples
- [ ] Document each dataset in `examples/README.md`
- [ ] Create ingestion scripts for each dataset

**Acceptance Criteria**:
- At least 3 complete example datasets
- Each dataset has 10+ documents
- README explains dataset purpose and usage
- Ingestion scripts tested and working

**Estimated Effort**: 1 day

## Short Term (1-2 Weeks)

### 4. Tutorial Documentation

**Status**: Not Started

**Tasks**:
- [ ] Write `documentation/tutorial-01-quickstart.md`
  - Installation
  - First ingestion
  - First search
  - Understanding results
- [ ] Write `documentation/tutorial-02-web-ui.md`
  - Starting the server
  - Using the web interface
  - Exploring statistics
- [ ] Write `documentation/tutorial-03-advanced.md`
  - Chunking strategies
  - Configuration options
  - Performance tuning
  - Troubleshooting
- [ ] Add tutorial links to README

**Acceptance Criteria**:
- Tutorials tested by following step-by-step
- Screenshots included where helpful
- Common issues documented
- Linked from main README

**Estimated Effort**: 2 days

### 5. Performance Benchmarks

**Status**: Not Started

**Tasks**:
- [ ] Create `benches/` directory with criterion benchmarks
- [ ] Benchmark ingestion pipeline:
  - Documents of varying sizes (1KB, 10KB, 100KB, 1MB)
  - Fixed vs semantic chunking
  - Batch sizes
- [ ] Benchmark search performance:
  - Database sizes (100, 1k, 10k, 100k chunks)
  - Query complexity
  - Top-K variations
- [ ] Benchmark vector similarity computation
- [ ] Document results in `documentation/performance.md`
- [ ] Add performance targets to CI (if exceeds baseline, fail)

**Acceptance Criteria**:
- Comprehensive benchmark suite
- Baseline performance documented
- Regression detection in CI
- Performance guide for users

**Estimated Effort**: 3 days

### 6. Release Preparation

**Status**: Not Started

**Tasks**:
- [ ] Create CHANGELOG.md with v0.1.0 changes
- [ ] Version bump in Cargo.toml (if needed)
- [ ] Create GitHub release template
- [ ] Build release binaries:
  - macOS (aarch64-apple-darwin)
  - macOS (x86_64-apple-darwin)
  - Linux (x86_64-unknown-linux-gnu)
  - Linux (aarch64-unknown-linux-gnu)
  - Windows (x86_64-pc-windows-msvc)
- [ ] Test each binary on target platform
- [ ] Create installation scripts for each platform
- [ ] Update README with installation instructions

**Acceptance Criteria**:
- Binaries built and tested for all platforms
- CHANGELOG complete
- Installation tested on each platform
- Release notes drafted

**Estimated Effort**: 3 days

## Medium Term (1-2 Months)

### 7. CI/CD Pipeline

**Status**: Not Started

**Priority**: High

**Tasks**:
- [ ] Create `.github/workflows/ci.yml`:
  - Run on: push, pull request
  - Jobs: format, clippy, test, build
  - Matrix: macOS, Linux, Windows
  - Cache: cargo dependencies
- [ ] Create `.github/workflows/release.yml`:
  - Trigger on: tag push (v*)
  - Build binaries for all platforms
  - Create GitHub release
  - Upload artifacts
- [ ] Create `.github/workflows/docs.yml`:
  - Build and deploy WASM demo to GitHub Pages
  - Run on: push to main
- [ ] Add status badges to README

**Acceptance Criteria**:
- All CI checks passing
- Automated release process working
- GitHub Pages auto-deploy
- Badges visible in README

**Estimated Effort**: 2 days

**Blockers**: None

### 8. PDF Support

**Status**: Not Started

**Priority**: High

**Tasks**:
- [ ] Research PDF parsing libraries (`pdf-extract`, `lopdf`, `pdfium`)
- [ ] Write tests for PDF ingestion (Red phase)
- [ ] Implement PDF text extraction (Green phase)
- [ ] Handle PDF metadata extraction
- [ ] Support page-level chunking option
- [ ] Refactor refactoring for clean integration
- [ ] Update CLI to accept .pdf files
- [ ] Add PDF examples to `examples/`
- [ ] Document PDF support in README

**Acceptance Criteria**:
- Can ingest PDF documents
- Text extraction quality verified
- Metadata preserved
- Tests passing
- Examples provided

**Estimated Effort**: 2-3 days

**Blockers**: None

### 9. SQLite-vec Integration

**Status**: Not Started

**Priority**: High

**Impact**: 10-100x faster search for large datasets

**Tasks**:
- [ ] Research sqlite-vec extension integration
- [ ] Evaluate alternatives (sqlite-vss, custom extension)
- [ ] Design migration strategy from BLOB to vec0 virtual table
- [ ] Write migration script with rollback
- [ ] Implement vector indexing (HNSW or IVF)
- [ ] Update search query to use vec0
- [ ] Benchmark before/after performance
- [ ] Create opt-in flag (keep BLOB as fallback)
- [ ] Document performance improvements

**Acceptance Criteria**:
- Search performance 10x+ faster for 10k+ chunks
- Migration tested with real data
- Backward compatibility maintained
- Performance documented

**Estimated Effort**: 1 week

**Blockers**:
- sqlite-vec may require custom compilation
- Need to verify cross-platform compatibility

### 10. Hybrid Search

**Status**: Not Started

**Priority**: Medium

**Tasks**:
- [ ] Research hybrid search algorithms (BM25 + vector)
- [ ] Implement BM25 scoring for keyword search
- [ ] Design score fusion strategy (RRF or weighted sum)
- [ ] Add `--hybrid` flag to search command
- [ ] Write tests for hybrid ranking
- [ ] Benchmark hybrid vs pure vector search
- [ ] Document when to use hybrid search

**Acceptance Criteria**:
- Hybrid search option available
- Performs better than pure vector on keyword queries
- Configurable weights
- Well-documented

**Estimated Effort**: 1 week

**Blockers**: None

## Long Term (3-6 Months)

### 11. Multi-Model Support

**Status**: Planned

**Priority**: Medium

**Description**: Allow different embedding models for different corpora within same database.

**Tasks**:
- [ ] Design schema changes (add model column to documents table)
- [ ] Update ingestion to track model per document
- [ ] Update search to filter by model or merge results
- [ ] Add `--model` flag to all commands
- [ ] Create migration for existing databases
- [ ] Document multi-model workflows

**Estimated Effort**: 1 week

### 12. Batch Operations

**Status**: Planned

**Priority**: Low

**Tasks**:
- [ ] Implement `vectdb batch-delete --source <pattern>`
- [ ] Implement `vectdb re-index --all`
- [ ] Implement `vectdb vacuum` (optimize database)
- [ ] Add progress bars for long operations
- [ ] Add `--dry-run` flag for safety

**Estimated Effort**: 3 days

### 13. Advanced Chunking Strategies

**Status**: Planned

**Priority**: Medium

**Tasks**:
- [ ] Markdown-aware chunking (preserve headers, code blocks)
- [ ] Code-aware chunking (function/class boundaries)
- [ ] Configurable chunking plugins
- [ ] Document strategy selection guide

**Estimated Effort**: 1 week

### 14. Packaging & Distribution

**Status**: Planned

**Priority**: Medium

**Tasks**:
- [ ] Create Homebrew formula (wrightmikea/homebrew-vectdb)
- [ ] Publish to crates.io
- [ ] Create Docker image
- [ ] Create Arch Linux AUR package
- [ ] Create Debian/Ubuntu .deb package
- [ ] Document all installation methods

**Estimated Effort**: 1 week

### 15. Community Engagement

**Status**: Planned

**Priority**: Low (but important for adoption)

**Tasks**:
- [ ] Enable GitHub Discussions
- [ ] Create CONTRIBUTING.md
- [ ] Create issue templates
- [ ] Create PR template
- [ ] Share on Reddit (r/rust, r/MachineLearning)
- [ ] Share on Hacker News
- [ ] Write blog post / dev.to article
- [ ] Create video tutorial (YouTube)

**Estimated Effort**: Ongoing

## Future Considerations (6-12 Months)

### Plugin Architecture

Allow custom embedding providers beyond Ollama:
- OpenAI embeddings
- Cohere embeddings
- HuggingFace local models
- Custom embedding functions

### Document Versioning

Track document changes over time:
- Version history
- Diff viewing
- Rollback capability

### Advanced Analytics

Usage and performance metrics:
- Query analytics
- Performance monitoring
- Usage patterns
- Cost tracking (if using paid models)

### Distributed Mode (Maybe)

Optional multi-node support for very large datasets:
- Sharding strategy
- Query routing
- Replication
- Consensus protocol

**Note**: This is speculative and may not align with "local-first" philosophy.

## Non-Goals

These features are explicitly **not** planned:

- Cloud-hosted version (violates local-first principle)
- Built-in LLM/chat interface (use external LLMs)
- Real-time collaboration
- Mobile apps
- Commercial licensing or support

## Risk Management

### Technical Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| SQLite-vec incompatible | High | Low | Keep BLOB fallback |
| PDF parsing unreliable | Medium | Medium | Clear documentation of limitations |
| Performance doesn't scale | High | Low | Benchmarking early |
| Ollama API changes | Medium | Low | Version pinning, graceful degradation |

### Project Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Low adoption | Medium | Medium | Good documentation, examples, tutorials |
| Competitor emerges | Low | Medium | Focus on local-first niche |
| Maintenance burden | High | Medium | Good architecture, tests, docs |

## Success Metrics

### v0.1.0 Launch (Next 2 Weeks)

- [ ] All Phase 7 documentation complete
- [ ] Release binaries available
- [ ] Live demo accessible
- [ ] 10+ GitHub stars
- [ ] Zero critical bugs

### v0.2.0 (2 Months)

- [ ] PDF support released
- [ ] SQLite-vec integrated
- [ ] CI/CD pipeline operational
- [ ] 50+ GitHub stars
- [ ] 5+ community contributions

### v1.0.0 (6 Months)

- [ ] All planned features complete
- [ ] 100+ GitHub stars
- [ ] Published on crates.io
- [ ] Featured in Rust newsletter or blog
- [ ] Active community (discussions, issues, PRs)

## How to Use This Plan

1. **Weekly Review**: Review this plan every Monday
2. **Prioritize**: Focus on immediate and short-term items first
3. **Update Status**: Mark tasks as complete, in progress, or blocked
4. **Adjust**: Adapt based on feedback and new information
5. **Communicate**: Keep README and status.md in sync

## Next Actions

**This Week**:
1. Complete documentation suite (finish README/CLAUDE.md updates)
2. Run full quality audit
3. Create example datasets

**Next Week**:
1. Write tutorial documentation
2. Begin performance benchmarking
3. Start release preparation

**This Month**:
1. Release v0.1.0
2. Set up CI/CD
3. Begin PDF support implementation

## Questions & Decisions Needed

1. **SQLite-vec timing**: Should we integrate before or after v0.1.0?
   - **Recommendation**: After v0.1.0 (in v0.2.0)
   - **Rationale**: Don't delay initial release

2. **Packaging priority**: Which package manager first?
   - **Recommendation**: Homebrew (macOS users), then crates.io
   - **Rationale**: Development platform + Rust community

3. **Video tutorials**: Worth the time investment?
   - **Recommendation**: Yes, after v0.1.0 release
   - **Rationale**: Helps adoption, but not critical for launch

## References

- [Product Requirements](./prd.md)
- [Architecture](./architecture.md)
- [Development Process](./process.md)
- [Project Status](./status.md)
