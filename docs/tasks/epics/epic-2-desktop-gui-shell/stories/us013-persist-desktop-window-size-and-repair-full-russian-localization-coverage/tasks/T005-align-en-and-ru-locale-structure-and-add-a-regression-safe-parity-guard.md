# T005: Align EN and RU locale structure and add a regression-safe parity guard

**Status:** Done
**Story:** US013
**Linear:** KGS-260

## Outcome

- EN and RU locale objects now share one explicit key structure.
- A parity check runs at module load so missing Russian keys stop being silent drift.
