all:
	bun kli.ts repo new -n Happy_Repo -p

a +ARGS:
	bun kli.ts repo new {{ARGS}}

b:
	bun build ./kli.ts --outdir /bin --target node

c:
	bun build ./kli.ts --outfile kli --compile
