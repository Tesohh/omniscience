#import "/resources/typst/lib/omni.typ": omni


#let note(
  title: "",
  tags: (),
  names: (),
  private: true,
  body,
) = {
  show: omni.with(title: title, tags: tags, names: names, private: private)

  // your custom template here

  body
}
