import { type Command, program } from 'commander'
import path from 'path'
import { promises as fs } from 'fs'
import { execSync } from 'child_process'
import toml from 'toml'
import os from 'os'
import { sleep, spawn } from 'bun'
import inquirer from 'inquirer'

// bun supports: top level await, jsx, and extensioned ts imports
// ts does not allow by default.
// recommended compilerOptions

// check config file exists
// N2H if not create it with prompts for username,

const CONFIG_PATH = path.join(os.homedir(), '.config/kli/config.toml')

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

interface KliConfig {
  kitty_session: string
  workspace?: {
    directory?: string
  }
  scripts?: Record<string, string>
}

async function main() {
  try {
    await checkConfig()
    const config = await readConfig()

    program.name('kli').description("Kenny's CLI").version('0.2.0')

    create_repo_commands(program, config)
    create_tinker_commands(program)
    create_ergo_command(program)
    create_ide_command(program)

    program
      .command('config')
      .description('Open KLI config file in editor')
      .action(() => {
        execSync(`${process.env.EDITOR || 'nvim'} ${CONFIG_PATH}`, {
          stdio: 'inherit',
        })
      })

    program
      .command('help')
      .description('Display help information')
      .action(() => {
        program.outputHelp()
      })

    await program.parseAsync(process.argv)
  } catch (error) {
    console.error(`An error occurred:`, error)
    process.exit(1)
  }
}

async function readConfig(): Promise<Config> {
  const configFile = await fs.readFile(CONFIG_PATH, 'utf-8')
  return toml.parse(configFile) as Config
}

function create_repo_commands(program: Command, config: Config) {
  let repo = program.command('repo')

  repo
    .command('new')
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
        try {
          const fileContent = await fs.readFile(filepath, 'utf-8')
          const file = JSON.parse(fileContent)
          if (!file) {
            console.error('Error: no repo name available in a package.json')
            process.exit(1)
          }
          repoName = file.name
        } catch (error) {
          console.error(`Error reading package.json:`, error)
          process.exit(1)
        }
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

  repo
    .command('del')
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

      let tinkerDir = path.join(os.homedir(), 'scratch', 'tinker', tinkerWord)

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
          console.log(`tinkering with ${tinkerWord} is not implemented`)
          process.exit(1)
      }

      try {
        await fs.access(tinkerDir)
        console.log(`Opening existing ${tinkerWord} project in ${tinkerDir}`)
      } catch (error) {
        console.log(`Creating new ${tinkerWord} project in ${tinkerDir}`)
        await fs.mkdir(tinkerDir, { recursive: true })
      }

      let windowId = await createKittyWindow(tinkerDir)

      try {
        await fs.access(path.join(tinkerDir, defaultFile))
      } catch (error) {
        await sendCommand(windowId, initCommand)
        await sendCommand(windowId, `touch ${tinkerDir}/${defaultFile}`)
      }

      await sendCommand(windowId, `nvim ${defaultFile}`)
    })
}

function create_ergo_command(program: Command) {
  program
    .command('ergo')
    .description('Interactively set up project ergonomics')
    .action(async () => {
      try {
        const config = await interactiveConfig()
        await saveConfig(config)
        console.log('Project ergonomics set up successfully!')
      } catch (error) {
        console.error('Error setting up project ergonomics:', error)
      }
    })
}

function create_ide_command(program: Command) {
  program
    .command('ide')
    .description('Start project IDE layout')
    .action(async () => {
      const configPath = path.join(process.cwd(), '.kli')
      try {
        const configContent = await fs.readFile(configPath, 'utf-8')
        const config: KliConfig = JSON.parse(configContent)

        const projectDir = config.workspace?.directory || process.cwd()
        const sessionConfigPath = await createKittySessionConfig(
          config.kitty_session,
          projectDir
        )

        const kittyCommand = `kitty --session ${sessionConfigPath}`
        execSync(kittyCommand)

        if (config.scripts?.startup) {
          console.log('Running startup script:', config.scripts.startup)
          execSync(config.scripts.startup, { cwd: projectDir })
        }
      } catch (error) {
        console.error('Error launching IDE', error)
      }
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

async function interactiveConfig(): Promise<KliConfig> {
  const questions = [
    {
      type: 'list',
      name: 'kitty_session',
      message: 'Select the Kitty session type:',
      choices: ['term', 'edserve'],
      default: 'term',
    },
    {
      type: 'input',
      name: 'workspace.directory',
      message:
        'Enter the workspace directory (leave empty for current directory):',
      default: '',
    },
    {
      type: 'confirm',
      name: 'addStartupScript',
      message: 'Do you want to add a startup script?',
      default: false,
    },
    {
      type: 'input',
      name: 'scripts.startup',
      message: 'Enter the startup script command:',
      when: (answers: any) => answers.addStartupScript,
    },
  ]

  const answers = await inquirer.prompt(questions)

  const config: KliConfig = {
    kitty_session: answers.kitty_session,
    workspace: {},
    scripts: {},
  }

  if (answers.workspace.directory) {
    config.workspace!.directory = answers.workspace.directory
  }

  if (answers.addStartupScript && answers.scripts.startup) {
    config.scripts!.startup = answers.scripts.startup
  }

  return config
}

async function saveConfig(config: KliConfig): Promise<void> {
  const configPath = path.join(process.cwd(), '.kli')
  await fs.writeFile(configPath, JSON.stringify(config, null, 2))
  console.log(`Configuration saved to ${configPath}`)
}

async function createKittySessionConfig(
  sessionType: string,
  projectDir: string
): Promise<string> {
  const configDir = path.join(os.homedir(), '.config', 'kitty', 'sessions')
  await fs.mkdir(configDir, { recursive: true })

  const projectName = path.basename(projectDir)
  const configPath = path.join(configDir, `${sessionType}.conf`)

  let config = ''
  switch (sessionType) {
    case 'edserve':
      config = `layout fat
cd ${projectDir}
launch --title "${projectName}.editor" zsh
launch --title "${projectName}.server" zsh
resize_window shorter 10`
      break
    case 'term':
      config = `cd ${projectDir}
launch zsh`
      break
    default:
      throw new Error(`Unsupported session type: ${sessionType}`)
  }

  await fs.writeFile(configPath, config)
  return configPath
}

async function checkConfig() {
  try {
    await fs.access(CONFIG_PATH)
    console.log(`Config file found`)
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
      console.log(`Config file not found. Creating a default one...`)
      const defaultConfig = `

[ssh]
username = ""
host = ""
nickname = ""

[github]
username = ""

[subdomain]
vhost_config = ""
domain = ""
`
      try {
        await fs.mkdir(path.dirname(CONFIG_PATH), { recursive: true })
        await fs.writeFile(CONFIG_PATH, defaultConfig)
        console.log('Default config file created.')
      } catch (writeError) {
        console.error('Error creating default config file:', writeError)
      }
    } else {
      console.error(`Error checking config file`, error)
      throw error
    }
  }
}

// use this for subdom action
// const vhostFile = fs.readFileSync('vhhost.tmpl.txt')

// Todo: subdom-new
// Todo: web-new
main().catch((error) => {
  console.error(`Unhandled error`, error)
  process.exit(1)
})
