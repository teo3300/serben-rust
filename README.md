# serben-rust

A really simple http server with mimnimal capabilities for serving files

## Working

Serve resources in the `content/` folder
- `/` requests `index.html`
- **html** files can be requested with or without the extension `html`
- **directories** list their content UwU (note that if you have `somename.html` and `somename/*` in the same level, the directory will have precedence, sorry about that)
- **`/*`** list `content/`'s content
- Files with extension `html`, `txt`, `css`, `js`, `xml` and `rss` are treated as text file
- Everything else is treated as binary files
- appending `.thumbnail` to a resource will fetch a thumbnail instead (dunno what happens if you fetch the thumbnail of something that isn't an image, ask `convert`)
- appending `.source` will force to load the file as text

## Installation and running
### Bare metal
The serving path must be specified when running bare metal
> Build and run the project from within the repo
```sh
mkdir content
cargo run --release -- <serving_directory>/
```
Or do some other fancy stuff, I am not stopping you

### Docker
When running in docker the path is fixed to `/content/` and you must mount your directory as a volume
> use the template docker compose to build the image and run the container
```sh
cp docker-compose.yml ..
cd ..
mkdir content
docker-compose up -d
```
(In this case use the parent directory as the project directory for docker).

**Manually create the "content" directory** to avoid ownership problems when editing files, not necessary, but highly recommended

I use traefik for routing and certificate management, on the network `${USER}_frontend` so be sure to set this properly

(I should put this in a Makefile but naaaaah)

Alternatively do fancy stuff

## convert

I really hope `convert` doesn't have any vulnerability