# TODO

- Improve error support
- Add async APIs
- A persistence implementation for:
  - SQL:
    - MySql
    - MariaDB
    - MSSQL
    - SQLite
  - NoSQL:
    - MongoDB
    - DynamoDb
    - Redis
- Inherited notes:
  - Event upcasters.
  - Event serialization uses the event type as the root node of the JSON tree.
    This simplifies deserialization but is non-standard.
