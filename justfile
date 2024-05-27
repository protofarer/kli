all:
	bun main.ts repo new -n Happy_Repo -p

a +ARGS:
	bun main.ts repo new {{ARGS}}
