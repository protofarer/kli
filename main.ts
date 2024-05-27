import { program } from "commander"
import path from 'path'
import fs from 'fs'
import { execSync } from 'child_process'

// bun supports: top level await, jsx, and extensioned ts imports 
// ts does not allow by default.
// recommended compilerOptions

// TODO: read in config
// Config { path: string, gh_username?: string }
const gh_username = 'protofarer'

program.name('kli').description('Kenny\'s utility belt CLI').version('0.2.0');

const repo = program.command('repo')
repo.command('new')
	// .argument("<string>", "Enter foo string")
	.option('-n, --name <string>', "Repository name")
	.option('-p, --public', "Create a public repository", false) // shorthand doesnt work
	.action(async (options) => {
		let repoName = options.name
		let isPublic = options.public || false

		// check for existing repo
		try {
			const isInsideWorkTree = execSync('git rev-parse --is-inside-work-tree').toString().trim();
			if (isInsideWorkTree === 'true') {
				const remoteOutput = execSync('git remote -v').toString();
				if (remoteOutput.includes('origin')) {
					console.error('Error: Remote repository already exists');
					process.exit(1);
				}
			} else {
				console.log('No local repo detected, creating one for you...');
				execSync('git init');
			}
		} catch (error) {
			console.error(error)
			process.exit(1)
		}

		if (!repoName) {
			const filepath = path.join(process.cwd(), 'package.json')
			const file = await JSON.parse(fs.readFileSync(filepath, 'utf-8'))
			if (!file) {
				console.error("Error: no repo name available in a package.json")
				process.exit(1)
			}
			repoName = file.name
		}

		console.log(`Attempting to create new repo ${repoName}`);

		try {
			execSync(`gh repo create ${repoName} ${isPublic ? '--public' : '--private'}`);
		} catch (error) {
			console.error(error);
			process.exit(1);
		}

		const url = `https://github.com/${gh_username}/${repoName}`;

		try {
			execSync(`git remote add origin ${url}`);
		} catch (error) {
			console.error(error);
			process.exit(1);
		}

		console.log(`Successfully created remote repo ${repoName} @ ${url}`);
	})


program.parse();

// repo.command('del')
// 	.description('Delete a remote repository')
// 	// .argument('<string>', 'Repository name')
// 	.option('-r, --repo <string>', 'Repository name')
// 	.action((_arg, options) => {
// 		console.log(`Delete this repo: ${options.repo}`)
// 	})

// Todo: subdom-new
// Todo: web-new
