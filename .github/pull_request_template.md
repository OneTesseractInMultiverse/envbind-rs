## Change

Describe the behavior, documentation, or release metadata change.

## Validation

List the commands you ran.

Run packaging commands after committing the reviewed changes to a clean tree.

```sh
make verify
make package-list
make package
```

## Checklist

- [ ] The change is focused on one concern.
- [ ] Public API changes appear in README and docs.
- [ ] Tests are self-contained and use one assertion per test.
- [ ] Built-in diagnostics omit raw values, and custom validation messages are safe to disclose when sensitivity is disabled.
- [ ] New dependencies are explained in the pull request.
