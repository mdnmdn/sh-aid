import OpenAI from 'openai';
import Anthropic from '@anthropic-ai/sdk';
import { GoogleGenerativeAI } from '@google/generative-ai';
import { $ } from "bun";
import os from "os";
import fs from "fs";
import path from "path";
import envPaths from "env-paths";

type ProviderType = "OpenAI" | "Custom" | "Claude" | "Gemini";

interface Config {
    type: ProviderType;
    apiKey?: string;
    model: string;
    baseURL?: string;
}

const DEFAULT_CONFIG: Config = {
    type: "OpenAI",
    model: "gpt-4.1",
};

function getConfig(): Config {
    const paths = envPaths('uwu', { suffix: '' });
    const configPath = path.join(paths.config, "config.json");

    if (!fs.existsSync(configPath)) {
        try {
            // If the config file doesn't exist, create it with defaults.
            fs.mkdirSync(paths.config, { recursive: true });
            const defaultConfigToFile = {
                ...DEFAULT_CONFIG,
                apiKey: "",
                baseURL: null,
            };
            fs.writeFileSync(
                configPath,
                JSON.stringify(defaultConfigToFile, null, 2)
            );

            // For this first run, use the environment variable for the API key.
            // The newly created file has an empty key, so subsequent runs will also fall back to the env var until the user edits the file.
            return {
                ...DEFAULT_CONFIG,
                apiKey: process.env.OPENAI_API_KEY,
            };
        } catch (error) {
            console.error("Error creating the configuration file at:", configPath);
            console.error("Please check your permissions for the directory.");
            process.exit(1);
        }
    }

    try {
        const rawConfig = fs.readFileSync(configPath, "utf-8");
        const userConfig = JSON.parse(rawConfig);

        // Merge user config with defaults, and also check env for API key as a fallback.
        return {
            ...DEFAULT_CONFIG,
            ...userConfig,
            apiKey: userConfig.apiKey || process.env.OPENAI_API_KEY,
        };
    } catch (error) {
        console.error("Error reading or parsing the configuration file at:", configPath);
        console.error("Please ensure it is a valid JSON file.");
        process.exit(1);
    }
}

const config = getConfig();

// The rest of the arguments are the command description
const commandDescription = process.argv.slice(2).join(' ').trim();

if (!commandDescription) {
    console.error("Error: No command description provided.");
    console.error("Usage: uwu <command description>");
    process.exit(1);
}

async function generateCommand(config: Config, commandDescription: string): Promise<string> {
    // Build the environment context
    const envContext = `
Operating System: ${os.type()} ${os.release()} (${os.platform()} - ${os.arch()})
Node.js Version: ${process.version}
Shell: ${process.env.SHELL || "unknown"}
Current Working Directory: ${process.cwd()}
Home Directory: ${os.homedir()}
CPU Info: ${os.cpus()[0]?.model} (${os.cpus().length} cores)
Total Memory: ${(os.totalmem() / 1024 / 1024).toFixed(0)} MB
Free Memory: ${(os.freemem() / 1024 / 1024).toFixed(0)} MB
`;

    // Get `ls -l` output (handle potential errors)
    let lsResult = "";
    try {
        lsResult = await $`ls`.text();
    } catch (error) {
        // If ls fails, provide fallback information
        lsResult = "Unable to get directory listing";
    }

    // System prompt
    const systemPrompt = `
You live in a developer's CLI, helping them convert natural language into CLI commands. 
Based on the description of the command given, generate the command. Output only the command and nothing else. 
Make sure to escape characters when appropriate. The result of \`ls -l\` is given with the command. 
This may be helpful depending on the description given. Do not include any other text in your response, except for the command.
Do not wrap the command in quotes.

--- ENVIRONMENT CONTEXT ---
${envContext}
--- END ENVIRONMENT CONTEXT ---

Result of \`ls -l\` in working directory:
${lsResult}
`;

    if (!config.apiKey) {
        console.error("Error: API key not found.");
        console.error("Please provide an API key in your config.json file or by setting the OPENAI_API_KEY environment variable.");
        process.exit(1);
    }

    switch (config.type) {
        case "OpenAI":
        case "Custom": {
            const openai = new OpenAI({
                apiKey: config.apiKey,
                baseURL: config.baseURL,
            });
            const response = await openai.chat.completions.create({
                model: config.model,
                messages: [
                    { role: "system", content: systemPrompt },
                    { role: "user", content: `Command description: ${commandDescription}` },
                ],
            });
            return response?.choices[0]?.message?.content?.trim() || "";
        }

        case "Claude": {
            const anthropic = new Anthropic({ apiKey: config.apiKey });
            const response = await anthropic.messages.create({
                model: config.model,
                system: systemPrompt,
                max_tokens: 1024,
                messages: [
                    { role: "user", content: `Command description: ${commandDescription}` },
                ],
            });
            // @ts-ignore
            return response.content[0]?.text.trim() || "";
        }

        case "Gemini": {
            const genAI = new GoogleGenerativeAI(config.apiKey);
            const model = genAI.getGenerativeModel({ model: config.model });
            const prompt = `${systemPrompt}\n\nCommand description: ${commandDescription}`;
            const result = await model.generateContent(prompt);
            const response = await result.response;
            return response.text().trim();
        }

        default:
            console.error(`Error: Unknown provider type "${config.type}" in config.json.`);
            process.exit(1);
    }
}

// --- Main Execution ---
try {
    const command = await generateCommand(config, commandDescription);
    console.log(command);
} catch (error: any) {
    console.error("Error generating command:", error.message);
    process.exit(1);
}