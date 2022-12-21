## Chatoy Server

Here is the server of Chatoy.

Written in Rust with love 🙌

### Perform the migrations

```bash
DATABASE_URL="sqlite:./data.db?mode=rwc" sea-orm-cli migrate refresh
```

### Generate entity from database

```bash
sea-orm-cli generate entity \
  -u "sqlite:./data.db?mode=rwc" \
  -o src/entities
```
