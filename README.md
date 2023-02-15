# climbsheet

<img width="1451" alt="image" src="https://user-images.githubusercontent.com/11027/219144813-7d8490d5-a338-4fdb-bd45-6b1657748286.png">

See `src/config.rs` for some documentation on configuration.

## adding a new gym

1. Add vertical life gym id to `config.toml`. The numeric id can be get by
   listening to Vertical Life iOS app's traffic with Charles proxy.

2. Create a new tab/sheet in the spreadsheet, for example, "Ristikko - Reitit".
   The prefix should match to the Kiipeilyareena location's name in Vertical
   Life API. For instance, if the gym's name in the API is "Kiipeilyareena
   Ristikko", then "Ristikko" is the prefix. This is best done by duplicating an
   existing tab.

3. Due to challenges with the sheets API, the sheet should not be completely
   empty when the table is filled for the first time. The problem is that API
   request that is used to append a row will try to insert to row so that it
   aligns with the header and the header does not start from the first column.
   That means that if you try to append to an empty table (with header only) the
   row will be aligned wrong relative to columns. The fix is to leave one row
   after duplicating and remove the extra row later. Hopefully the program will
   be able to automatically remove non-existent rows soon.
