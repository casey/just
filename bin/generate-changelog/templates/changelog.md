Changelog
=========
{% for release in releases %}
{{ release.header(format) }}
{% for (ty, section) in release.sections %}
### {{ ty }}
{% for commit in section.commits -%}
- {{ commit.entry(format) }}
{% endfor -%}
{% endfor -%}
{% endfor -%}
