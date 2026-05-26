# PR Review Checklist

## Scope

- [ ] One task only.
- [ ] Touched paths match lease.
- [ ] No drive-by refactors.

## Safety

- [ ] Trading semantics unchanged or golden trace updated.
- [ ] Precision behavior unchanged or dual precision tests added.
- [ ] No unsafe added, or unsafe is justified and tested.
- [ ] No adapter behavior changed without fixtures.

## Tests

- [ ] Targeted tests run.
- [ ] Fast verification run or blocker documented.
- [ ] Rust CLI smoke run if product surface touched.
- [ ] Golden trace run if behavior touched.

## Docs

- [ ] Public API changes documented.
- [ ] Migration note added if required.
- [ ] Evidence file complete.

## Release impact

- [ ] No release blocker introduced.
- [ ] Rollback plan clear.
