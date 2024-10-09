# Openmina Frontend

This is a simple Angular application that will help you to see the behaviour of your local rust based mina node.

## Prerequisites

### 1. Node.js v20.11.1

#### MacOS

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install node@20.11.1
```

#### Linux

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
source ~/.bashrc
nvm install 20.11.1
```

#### Windows

Download [Node.js v20.11.1](https://nodejs.org/) from the official website, open the installer and follow the prompts to complete the installation.

### 2. Angular CLI v16.2.0

```bash
npm install -g @angular/cli@16.2.0
```

### 3. Installation

Open a terminal and navigate to this project's root directory

```bash
cd PROJECT_LOCATION/openmina/frontend
```

Install the dependencies

```bash
npm install
```

## Run the application

```bash
npm start
```

# Using O1JS wrapper

as of now, o1js is not prepared to work with Angular, therefore we need to use the wrapper that is provided in the `src/assets/o1js` folder. This wrapper is a simple javascript webpack based application that will allow us to use the o1js library in our Angular application.

How to use it:

1. Open a terminal and navigate to the `src/assets/o1js` folder
2. Install the dependencies

```bash
npm install
```

3. Build the wrapper

```bash
npm run build-o1jswrapper
```

4. That's it. Now you can use your code from o1js-wrapper inside the Angular application by using `BenchmarksWalletsZkService => o1jsInterface`
