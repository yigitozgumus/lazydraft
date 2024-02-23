# lazydraft

Over-engineered CLI tool to transfer and modify my Obsidian notes to my website for publishing.

## Motivation

I usually write my content in Obsidian. Keeping everything in there makes sense for me but when I want to move something out of Obsidian to my website, I need to:

- Modify the content of the writing if I have any images or assets
- Find the related asset files and copy them as well

Doing this one time is okay, but If I don't like something and change it, now I need to do it in multiple places.

I want to simplify this process.

## How to install

You can use [Homebrew](https://brew.sh) to install lazydraft.

```bash
brew tap yigitozgumus/formulae
brew install lazydraft
```

## How to use

There are three commands.

- status
- stage
- config

The config command does not work at the moment. When you run it for the first time, it creates an empty config file ready to be filled by you. You can find more information in my accompanying [writing series](https://www.yigitozgumus.com/series/building-a-cli-in-rust/).

## Roadmap

Currently, the main functionality works. I will plan to:

- [x] Add some front matter sanitization to decrease further the likelihood of getting an unexpected error.
- [x] Remove empty frontmatter properties.
- [x] Add coverImage area to frontmatter (Adding it in Obsidian seems pointless)
- [x] Remove any Wikilink present in the Note. Expect the images. (This means that I can write stuff all the while linking to other stuff in Obsidian, but the formatting is handled automatically when I want to publish that thing)

