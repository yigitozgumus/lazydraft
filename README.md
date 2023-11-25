# lazydraft

Over engineered CLI tool to transfer and modify my Obsidian notes to my personal website for publishing.

## Motivation

I usually write my content in Obsidian. Keeping everything in there makes sense for me but when I want to move something out of Obsidian to my personal website, I need to:

- Modify the content of the writing if I have any images or assets
- Find the related asset files and copy them as well

Doing this one time is okay, but If I don't like something and change it, now I need to do it in multiple places.

I want to simplify this process.

## How to install

Download the binary to any folder that is in your `$PATH`. I prefer `~/.local/bin`

## How to use

There are three commands.

- status
- stage
- config

Config command does not work at the moment. When you run it for the first time, it creates an empty config file ready to be filled by you. You can find more information in my accompanied [writing series](https://www.yigitozgumus.com/series/building-a-cli-in-rust/).

## Roadmap

Currently the main functionality works. I will plan to add some frontmatter sanitization to further decrease of getting en unexpected error.

- [ ] Add coverImg area to frontmatter (Adding it in Obsidian seems pointless)
- [ ] Remove empty frontmatter properties.
