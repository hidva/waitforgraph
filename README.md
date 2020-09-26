A utility used for generating locks wait-for graph for Greenplum Database

```bash
# Use PG* environment variables (PGDATABASE, PGUSER, ...) as connection information by default
waitforgraph > wfg.dot
dot -T svg < wfg.dot > wfg.svg
# also support Connection String described in https://www.postgresql.org/docs/current/libpq-connect.html
waitforgraph dbname=DATABASENAME | dot -T svg > wfg.svg
open wfg.svg
```

The locks wait-for graph looks as follow:

![gdd.dot](https://raw.githubusercontent.com/hidva/waitforgraph/master/assets/wfg.svg)

There is more detailed information in the dot file:

```dot
strict digraph G {
label="WaitForGraph - Generated By hidva/waitforgraph";
29968;
29970;
29970 -> 29968
}
/*大吉大利~
session 29970 waits for AccessShareLock on locktype=relation,gp_segment_id=-1,database=10902,relation=16395; blocked by session 29968(granted AccessExclusiveLock);
*/
```
