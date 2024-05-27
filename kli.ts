import { program } from "commander"
import path from 'path'
import fs from 'fs'
import { execSync } from 'child_process'
import toml from 'toml'
import os from 'os'

// bun supports: top level await, jsx, and extensioned ts imports 
// ts does not allow by default.
// recommended compilerOptions

// check config file exists
// N2H if not create it with prompts for username, 

interface Config {
	ssh: {
		username: string;
		host: string;
		nickname: string;
	};
	github: {
		username: string;
	};
	subdomain: {
		vhost_config: string;
		domain: string;
	};
}

function readConfig() {
	const configPath = path.join(os.homedir(), '.config/kli/config.toml')
	const configFile = fs.readFileSync(configPath, 'utf-8')
	const config: Config = toml.parse(configFile)
	const ghUsername = config.github.username
	const domain = config.subdomain.domain
	return { ghUsername, domain }
}

function main() {
	const { ghUsername, domain } = readConfig()

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
				execSync('git rev-parse --is-inside-work-tree', { stdio: 'ignore' })
				const remoteOutput = execSync('git remote -v', { stdio: 'ignore' }).toString()
				if (remoteOutput.includes('origin')) {
					console.warn('Remote repository already exists');
					process.exit(1);
				}
			} catch (error) {
				console.error(error)
			}

			console.log("No local repo detected, creating one for you..")
			execSync('git init');

			if (!repoName) {
				const filepath = path.join(process.cwd(), 'package.json')
				const file = await JSON.parse(fs.readFileSync(filepath, 'utf-8'))
				if (!file) {
					console.error("Error: no repo name available in a package.json")
					process.exit(1)
				}
				repoName = file.name
			}

			console.log(`Creating new repo: "${repoName}"`);

			try {
				execSync(`gh repo create ${repoName} ${isPublic ? '--public' : '--private'}`);
			} catch (error) {
				console.error(error);
				process.exit(1);
			}

			const url = `https://github.com/${ghUsername}/${repoName}`;

			try {
				execSync(`git remote add origin ${url}`);
			} catch (error) {
				console.error(error);
				process.exit(1);
			}

			console.log(`Successfully created remote repo ${repoName} @ ${url}`);
		})

	repo.command('del')
		// .argument("<string>", "Enter foo string")
		.argument('<string>', 'Repository to delete')
		.action(async (repoName) => {
			try {
				execSync(`gh repo delete https://github.com/${ghUsername}/${repoName} --yes`)
			} catch (error) {
				console.error(error)
			}
			console.log(`Deleted repo ${repoName}`)
		})


	program.parse();
}

main()

// use this for subdom action
// const vhostFile = fs.readFileSync('vhhost.tmpl.txt')

// Todo: subdom-new
// Todo: web-new
