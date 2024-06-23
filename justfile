all:
	bun kli.ts repo new -n Happy_Repo -p

a +ARGS:
	bun kli.ts repo new {{ARGS}}

build-node:
	bun build ./kli.ts --outdir /bin --target node

compile:
	bun build src/kli.ts --outfile kli --compile
