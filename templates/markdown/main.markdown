## FontSpector report

fontspector version: {{version}}

{% if fatal_checks %}

## Checks with FATAL results

These must be addressed first.

{% for filename, checks in fatal_checks %}
{% include "checks.markdown" %}
{% endfor %}
{% endif %}
{% if experimental_checks %}

## Experimental checks

These won't break the CI job for now, but will become effective after some time if nobody raises any concern.

{% for filename, checks in experimental_checks %}
{% include "checks.markdown" %}
{% endfor %}
{% endif %}
{% if other_checks %}
{% if experimental_checks or fatal_checks %}

## All other checks

{% else %}

## Check results

{% endif %}

{% for filename, checks in other_checks %}
{% include "checks.markdown" %}
{% endfor %}
{% endif %}

{% if total > 0 %}

### Summary

| {%for level in summary_keys %}{{level | emoticon }} {{level}} | {%endfor%}
| {%for level in summary_keys %}---|{%endfor%}
| {%for level in summary_keys %}{{summary[level]}} | {%endfor%}
| {%for level in summary_keys %}{{summary[level] | percent(total=total)}} | {%endfor%}
{% endif %}

{% if omitted %}
**Note:** The following loglevels were omitted in this report:

{% for level in omitted %}

- {{level}}{% endfor %}
  {% endif %}
