# An engine for generating HTML files from templates
This all started with wanting a blogish website.
Determined to not pay for hosting, I settled upon a GitHub pages site.
I wanted to have a consistent theme, a sort of template for each article, but by virtue of GitHub pages serving static content, each post would need its own HTML page, with the skeleton copy-pasted in.
Therefore, if I wanted to change that theme, I would have to manually edit each HTML file.
Thus, this project was born.
Would Jekyll solve the exact same problem?
Sure, but I wanted to learn Rust anyway.

# How it works
The idea is that you feed in some collection of template files, article files, and static files, that a fully formed, ready to host, website is spit out.

Template files are stored in `/src/templates`.
The most basic template is just some text.
For example, heading.html contains the following
```html
<div id="headerRow">
    <a href="/" id="headerlink"><h1 id="header">VI</h1></a>
    <a id="archivelink" href="./archive.html">archive</a>
</div>
```

In an article, or another template file, any instances of `${heading}` in those files will expand to that text.

Some templates can contain other content. For example, a HTML skeleton template would want to contain some content in its body.

These are called reverse templates.
A template that wants content to be inserted into it can mark the position they want this done in by placing a `{-}` there.
You can optionally follow the dash with some text to label them, for example, `${-name}` conveys that the reverse template expects some name.
This label for the reverse template content is only for the human, the program doesn't care about it at all.

As an example, an article skeleton template may contain
```html
<body>
    <div id="main">
        ${heading}
        <hr>
        <div id="content">
            ${-content}
        </div>
    </div>
    <script src="./index.js" async defer></script>
</body>
```
Where `${heading}` will expand to the heading template, and any reverse template content will be inserted into the document by replacing `${-content}`

Finally, templates are static, and only expand to the text that is in the file.

If you want a dynamic template, like one that expands to the current date, a bang is to be used.

Bangs live in the template directory, but are called with an exclamation mark, so you would use `!{date}` to expand the date bang.

Bangs are defined in the program, as they need some code to be run.

All of these various expanding bits of text are strung together to produce an article.
Articles live in the articles directory, and may contain whatever bang and template calls they want.

# How to use it
After writing templates and bangs to build your articles, you use the command line interface to actually compile the documents.

There are two commands currently, `compile`, and `publish`.
The compile command will compile the template that comes after compile.
So calling `./engine compile article1` will compile the article in `/articles/article1.html`.

There is a special subcommand for compile that will compile all documents, which is `compile all`.

Finally, the `publish` command will `cd` into the built directory, and run `git add .`, `git commit`, and `git push`.

It's not very efficient, but that's what works for now.

# Should I use it?
Of course not.
I wrote this README so that I could look back it later.
However, if you are so inclined, I think it's a nice system. There are other, much more fully formed tools out there, though.