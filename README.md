# omniscience
bridging different PKMs together to create the ultimate PKM 

## what is this about? 
this project is an effort to bridge together different PKMs and note taking / typesetting formats 
(obsidian, org-roam, typst...) as each format has it's own strengths:

You might like the ease of use and mobile-friendlyness of obsidian, 
but you might also like the task management in org,
but at the same time you might also like typst for when you need the big guns... 

you might also like markdown as a format for blog posts...
but why not, you might want to use typst for that.. or org..

**So this project is about**:
1. Creating a single database of filenames / slugs / id's / aliases etc. for linking
2. Recognize "namespaced" links in all supported formats (eg. `[[omni:my-note]]` in markdown or `@omni.my-note` in typst)
3. Recognize format specific links (eg. `[[obs-note]]` in obsidian vaults)
4. Provide autocomplete and other assistances when creating pages through a language server
5. "Compile" all files into a single directory, with all links normalized (eg. `@omni.my-note` => `https://example.com/omni/my-note`)
6. Run these files through an SSG (provided by the user), like Hugo to get a nice "digital garden"

## personal reasons
i wanted to make this project as i am a hyper-configurer and have tried many different PKM systems,
and sometimes i want to try new ones (or even formats that don't "have" a pkm ie. typst), 
but that would mean migrating all old notes (SOMEHOW), or having disconnected systems.

all of them have their pros and cons:
- org{-roam}: has the best organization philosophy (hierarchical), but needs disgusting emacs, 
  which is just another thing i have to configure... I didn't really use the task management feature to be fair.
- obsidian: very easy to get started with and used it for quite a while. has no full latex support, 
  and requires a disgusting electron bloated mess of an editor for many advanced features, with bad vim bindings
  (yes i know about `obsidian.nvim`)
- typst: very versatile and delightful to write math in,  
  but doesn't have any way to link between pages (or well, not in the "zettelkasten" sense)

so ideally you should be able to use all of them at the same time, together.

