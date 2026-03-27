Reproduce by running `cargo test`

Note that the generated spec is parseable by redocly:

```shell
cargo run &
curl http://localhost:8080/v1/api.json > api.json
npx @redocly/cli build-docs api.json
open redoc-static.html
```
