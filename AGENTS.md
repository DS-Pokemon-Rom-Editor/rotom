# AGENTS.md

Behavioral guidelines for agents working on this project.

## DSPRE Reference Data

When working with DSPRE command databases, use the reference repository at [scrcmd-database](https://github.com/DS-Pokemon-Rom-Editor/scrcmd-database). Verify command names, IDs, parameter types, and parameter values against its legacy or v2 files before creating fixtures or making claims about the database.

## Game Engine Reference Data

This entire scripting engine compiles down to a single reference implementation: NDS pokemon field scripts. When working with scripts, verify behavior against the game engine whenever possible. For this, the platinum and hgss decompilations are available at [platinum](https://github.com/pret/pokeplatinum/) and [hgss](https://github.com/pret/pokeheartgold/) respectively. NEVER call anything engine related verified without having researched the actual in game implementation of a command, pattern or similar.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them; don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

NEVER MAKE ASSUMPTIONS YOU CAN NOT VERIFY AT THE SOURCE.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it; don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

**Clippy on touched Rust:** After substantive edits, run Clippy on the relevant crate(s), e.g. `cargo clippy -p rotom --all-targets` or `cargo clippy --workspace --all-targets` when multiple members change. Address **new** warnings in **files or modules you changed** (including lints reported for that code). 

**Coverage on new Rust tests:** When adding tests, verify the impact with `cargo llvm-cov`, e.g. `cargo llvm-cov --all-features --workspace --summary-only` for the workspace or `cargo llvm-cov -p rotom-lsp --all-features --summary-only` for one package.

## 4. Don't Reimplement Upstream Logic

**When a dependency already solves the problem, don't rewrite it.**

- Before adding evaluators, parsers, or resolvers in your crate, search the upstream crate's public API. It probably already has what you need.
- If an upstream sister crate's method is private, make it public or add a thin public seam there; never copy the logic into your crate.
- Don't round-trip through text (serialize → deserialize) when you have structured data. Pass the structures directly.

Ask yourself: "Is this logic already in the dependency's domain?" If yes, add the seam there, not here.

## 5. Parse Once, Reuse the AST

**The AST is the canonical representation. Don't re-derive it.**

- If you need to extract metadata (includes, defines, symbols) and also compile, do both from the same parsed AST.
- Don't parse the source for constants, then parse it again for codegen.
- Don't create intermediate text representations of the AST to feed to another parser.

## 6. Don't Break One Platform to Fix Another

**Platform-specific quirks belong in platform-specific code.**

- If a protocol change (LSP format, command name, argument shape) fixes editor A but breaks editor B, the fix is wrong.
- Fix the odd-one-out editor in its extension adapter, not in the shared protocol.
- The shared protocol should use the most standard/portable format. Adapters translate.

## 7. Consolidate, Don't Proliferate

**Remove cruft before adding new things.**

- If you see 5 functions that could be 3, simplify before extending.
- Don't add helper functions called exactly once. Inline them.
- **Never add a module or shared helper for a few lines** that are only used once or twice, especially when call sites still pass closures anyway and gain nothing. Duplicate the snippet inline until several real call sites justify extraction (see §9).
- When your changes make a module or function dead, delete it; don't leave it lying around.
- Before defining a new type, search the codebase. If an equivalent type already exists (same shape, same domain), use it directly. Wrapper types that only exist to rename another type are never acceptable.

## 8. Ask Before Building Infrastructure

**If you're about to add a new module, method, or abstraction, stop and ask.**

- If the approach requires re-reading files, building synthetic content, or duplicating existing logic, it's probably wrong.

## 9. Document What You Touch

**If a function lacks a doc comment and you modify it, add one.**

- Keep docs concise and in simple, easy to understand language.
- Surface-level APIs (public entry points, CLI plumbing, LSP handlers) deserve fuller docs:
  - One-line summary
  - Inputs / outputs
  - Errors or options where non-obvious
- Don't document the obvious (`/// Returns true` on `is_success`).

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, clarifying questions come before implementation rather than after mistakes, and touched Rust code is checked with Clippy without dumping unrelated cleanups into the same change.
