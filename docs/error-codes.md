# Error Codes

Python UX helpers raise `McConfigurationError` for user-facing configuration
problems.

| Code | Meaning | Suggested Fix |
| --- | --- | --- |
| `MC_CONFIG_PATHS` | `n_paths` is zero or negative. | Use a positive integer such as `100_000`. |
| `MC_CONFIG_STEPS` | `n_steps` is zero or negative. | Use a positive integer such as `64`. |
| `MC_CONFIG_POSITIVE` | A required positive model field is zero or negative. | Check `spot`, `strike`, and `maturity`. |
| `MC_CONFIG_VOLATILITY` | Volatility is negative. | Use zero or a positive decimal such as `0.2`. |
| `MC_CONFIG_BARRIER` | Down-and-out barrier is outside current helper assumptions. | Use `0 < barrier < spot`. |

Example:

```python
from montepath import EuropeanCallConfig, McConfigurationError, price_european_call

try:
    price_european_call(EuropeanCallConfig(n_paths=0))
except McConfigurationError as exc:
    print(exc.code)
    print(exc.suggestion)
```

