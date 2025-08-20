# Macro report

{#each group in groups }

## {{ group.name }}

| Macro   | Amount | RNI |
|---------|--------|-----|
| Protein | {{ sum(each group.recipes as r, "ingredients", r.name, ”"macronutrients.protein") }}| 100g |
....

{#each}
