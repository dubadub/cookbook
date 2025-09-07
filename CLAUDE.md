# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a recipe collection and meal planning system using the CookLang format, designed to work with the CookCLI tool. The repository contains structured recipes in `.cook` files organized by category, along with configuration for shopping lists, ingredient metadata, and meal planning.

## Menu preferences

For evenings should be something light like @./Prawn Evening Salad{} or @./Caprese{}, low carbs.

## Essential Commands

### Read a recipe
```sh
cook recipe read "Root Vegetable Tray Bake.cook"
```

### Generate shopping list
```sh
cook shopping-list \
  "Neapolitan Pizza.cook" \
  "Root Vegetable Tray Bake.cook" \
  "Snack Basket I.cook"
```

### Run local web server
```sh
cook server
```
Then open http://127.0.0.1:9080 in browser.

## Repository Structure

### Recipe Organization
- **By meal type**: `/Breakfast/`, `/Lunches/`, `/Dinners/`, `/Snacks/`, `/Sides/`
- **By cuisine**: `/French/`, `/Turkish/`, `/Russian New Year/`
- **By cooking method**: `/Slowcooker/`, `/Stir Fry/`, `/Traybake/`, `/Baking/`
- **By special purpose**: `/Components/` (reusable elements), `/Freezable/`, `/Plans/` (meal plans)

### Configuration System
- `/config/aisle.conf` - Shopping list organization by grocery store aisles
- `/config/units.yml` - Unit conversion definitions
- `/config/ingredients/*/` - Per-ingredient metadata including:
  - `market.yml` - Store prices
  - `nutrition_facts.yml` - Nutritional information
  - `physics.yml` - Physical properties
  - `tags.yml` - Categorization

## CookLang Syntax

### Basic Format
- **Ingredients**: `@ingredient{quantity%unit}` (e.g., `@garlic{3%cloves}`)
- **Equipment**: `#equipment{}` (e.g., `#mixing bowl{}`)
- **Time**: `~{duration}` (e.g., `~{2%hours}`)
- **Recipe references**: `@./path/to/recipe{}` for meal plans

## Meal Plan Format

Meal plans use a structured day-by-day format with markdown headers and CookLang syntax:

```
==Day 1==

Breakfast:
- @./Breakfast/Pancakes{3} with @maple syrup{2%tbsp} and @butter{1%tbsp}
- @coffee{1} with @cream{1%tbsp}

Lunch:
- @./Lunches/Cheeseburger{1} with @fries{1%portion}
- @cola{1%can}

Dinner:
- @nachos{1%plate} with @cheddar{1%cup}, @jalapeños{1%tbsp}
- @./Dinners/Chicken quesadilla{1}

==Day 2==
...
```

### Meal Plan Conventions
- **Day headers**: Use `==Day N==` format
- **Meal sections**: `Breakfast:`, `Lunch:`, `Dinner:` (can include other meals)
- **Recipe references**: `@./Category/Recipe Name{quantity}` for linking to recipe files
- **Direct ingredients**: `@ingredient{quantity%unit}` for simple additions
- **Combining**: Mix recipe references with direct ingredients using "with"

### YAML Frontmatter
Recipes can include metadata like:
```yaml
---
servings: 4
prep_time: 15 minutes
cook_time: 30 minutes
---
```

## Working with Recipes

### Recipe Categories
- Individual recipes are in category directories
- "Basket" files (e.g., `Fruit Basket I.cook`) are ingredient collection lists
- `/Plans/` contains meal planning files that reference multiple recipes
- `/Components/` contains reusable recipe elements like doughs and sauces

### Ingredient Management
When adding new ingredients, consider updating:
1. `/config/aisle.conf` for shopping list organization
2. Ingredient-specific configuration in `/config/ingredients/[ingredient-name]/`

### File Naming
- Use descriptive names with proper capitalization
- `.cook` extension for all recipe files
- Organize into appropriate category directories

## Integration Notes

This repository is designed to work with the CookCLI ecosystem for:
- Automated shopping list generation
- Web-based recipe browsing
- Meal planning and scheduling
- Nutritional analysis and reporting

Refer to [cooklang.org](https://cooklang.org/cli/help/) for comprehensive CookCLI documentation.
