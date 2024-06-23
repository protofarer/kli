import { type Command, program } from 'commander'
import path from 'path'
import fs from 'fs'
import { mkdir } from 'fs/promises'
import { execSync } from 'child_process'
import toml from 'toml'
import os from 'os'
import { sleep, spawn } from 'bun'

// bun supports: top level await, jsx, and extensioned ts imports
// ts does not allow by default.
// recommended compilerOptions

// check config file exists
// N2H if not create it with prompts for username,

interface Config {
    ssh: {
        username: string
        host: string
        nickname: string
    }
    github: {
        username: string
    }
    subdomain: {
        vhost_config: string
        domain: string
    }
}

function main() {
    let config = readConfig()

    program.name('kli').description("Kenny's CLI").version('0.2.0')

    create_repo_commands(program, config)
    create_tinker_commands(program)

    program.parse()
}

function readConfig() {
    let configPath = path.join(os.homedir(), '.config/kli/config.toml')
    let configFile = fs.readFileSync(configPath, 'utf-8')
    let config: Config = toml.parse(configFile)
    return config
}

function create_repo_commands(program: Command, config: Config) {
    let repo = program.command('repo')

    repo.command('new')
        // .argument("<string>", "Enter foo string")
        .option('-n, --name <string>', 'Repository name')
        .option('-p, --public', 'Create a public repository', false) // shorthand doesnt work
        .action(async (options) => {
            let repoName = options.name
            let isPublic = options.public || false

            // check for existing repo
            try {
                execSync('git rev-parse --is-inside-work-tree', {
                    stdio: 'ignore',
                })
                let remoteOutput = execSync('git remote -v', {
                    stdio: 'ignore',
                }).toString()
                if (remoteOutput.includes('origin')) {
                    console.warn('Remote repository already exists')
                    process.exit(1)
                }
            } catch (error) {
                console.error(error)
            }

            console.log('No local repo detected, creating one for you..')
            execSync('git init')

            if (!repoName) {
                let filepath = path.join(process.cwd(), 'package.json')
                let file = await JSON.parse(fs.readFileSync(filepath, 'utf-8'))
                if (!file) {
                    console.error(
                        'Error: no repo name available in a package.json'
                    )
                    process.exit(1)
                }
                repoName = file.name
            }

            console.log(`Creating new repo: "${repoName}"`)

            try {
                execSync(
                    `gh repo create ${repoName} ${isPublic ? '--public' : '--private'}`
                )
            } catch (error) {
                console.error(error)
                process.exit(1)
            }

            let url = `https://github.com/${config.github.username}/${repoName}`

            try {
                execSync(`git remote add origin ${url}`)
            } catch (error) {
                console.error(error)
                process.exit(1)
            }

            console.log(`Successfully created remote repo ${repoName} @ ${url}`)
        })

    repo.command('del')
        // .argument("<string>", "Enter foo string")
        .argument('<string>', 'Repository to delete')
        .action(async (repoName) => {
            try {
                execSync(
                    `gh repo delete https://github.com/${config.github.username}/${repoName} --yes`
                )
            } catch (error) {
                console.error(error)
            }
            console.log(`Deleted repo ${repoName}`)
        })
}

enum TinkerWord {
    Node = 'node',
    NodeTs = 'node-ts',
    Web = 'web',
    WebTs = 'web-ts',
    Bun = 'bun',
    Rust = 'rust',
    Odin = 'odin',
}

function create_tinker_commands(program: Command) {
    program
        .command('tinker <tinker_word>')
        .description('Start tinkering with a language or framework')
        .action(async (tinkerWord: string) => {
            if (!Object.values(TinkerWord).includes(tinkerWord as TinkerWord)) {
                console.error(
                    `Invalid tinker project word: ${tinkerWord}. Allowed values are: ${Object.values(TinkerWord).join(', ')}`
                )
                process.exit(1)
            }

            let tinkerDir = path.join(
                os.homedir(),
                'scratch',
                'tinker',
                tinkerWord
            )

            let initCommand = ''
            let defaultFile = ''

            switch (tinkerWord as TinkerWord) {
                case TinkerWord.Node:
                    initCommand = 'npm init -y'
                    defaultFile = 'index.js'
                    break
                case TinkerWord.NodeTs:
                    initCommand =
                        'npm init -y && npm install --save-dev typescript @types/node && npx tsc --init'
                    defaultFile = 'index.ts'
                    break
                case TinkerWord.Web:
                    initCommand = 'echo "Creating Web project"'
                    defaultFile = 'index.html'
                    break
                // case TinkerWord.WebTs:
                // initCommand =
                //     'npm init -y && npm install --save-dev typescript && npx tsc --init'
                // defaultFile = 'index.ts'
                // 	break
                case TinkerWord.Bun:
                    initCommand = 'bun init'
                    defaultFile = 'index.ts'
                    break
                case TinkerWord.Rust:
                    initCommand = 'cargo init'
                    defaultFile = 'src/main.rs'
                    break
                case TinkerWord.Odin:
                    initCommand =
                        'echo "package main\n\nimport \\"core:fmt\\"\n\nmain :: proc() {\n    fmt.println(\\"Hello, Odin!\\")\n}" > main.odin'
                    defaultFile = 'main.odin'
                    break
                default:
                    console.log(
                        `tinkering with ${tinkerWord} is not implemented`
                    )
                    process.exit(1)
            }

            if (!fs.existsSync(tinkerDir)) {
                console.log(
                    `Creating new ${tinkerWord} project in ${tinkerDir}`
                )
                mkdir(tinkerDir, { recursive: true })
            } else {
                console.log(
                    `Opening existing ${tinkerWord} project in ${tinkerDir}`
                )
            }

            let windowId = await createKittyWindow(tinkerDir)

            if (!fs.existsSync(path.join(tinkerDir, defaultFile))) {
                await sendCommand(windowId, initCommand)
                await sendCommand(windowId, `touch ${tinkerDir}/${defaultFile}`)
            }

            await sendCommand(windowId, `nvim ${defaultFile}`)
        })
}

async function sendCommand(windowId: string, command: string) {
    let p = spawn(
        [
            'kitty',
            '@',
            'send-text',
            '--match',
            `id:${windowId.trim()}`,
            command + '\n',
        ],
        { stdin: 'pipe' }
    )
    await p.exited
}

async function createKittyWindow(tinkerDir: string) {
    let kittyLaunchCmd = [
        'kitty',
        '@',
        'launch',
        '--type=os-window',
        '--cwd',
        tinkerDir,
    ]
    let kittyProcess = spawn(kittyLaunchCmd)
    let windowId = await new Response(kittyProcess.stdout).text()

    await sleep(1000)
    return windowId
}

// use this for subdom action
// const vhostFile = fs.readFileSync('vhhost.tmpl.txt')

// Todo: subdom-new
// Todo: web-new
main()
