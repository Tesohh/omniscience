#import "/resources/typst/templates/note.typ": note;

{% raw %}
#show: note.with(
  title: "{{ title }}",
  tags: (),
  names: ("{{ name }}",),
)
{% endraw %}

