<details>
    <summary>{{check.worst_status | emoticon}} <b>{{check.worst_status}}</b> {{check.check_name}} ({{check.check_id}})</summary>
    <div>

{% if not succinct %}
{% for line in check.check_rationale | split(pat="\n") %}> {{line | unindent | replace(from="\n", to="") }}
{% endfor %}
{% endif %}

{% if not succinct and proposals[check.check_id]%}
Original proposal: {{proposals[check.check_id]}}
{% endif %}

{% for result in check.subresults |sort(attribute="severity") %}
{% if not result is omitted %}

- {{result.severity | emoticon }} **{{result.severity}}** {% if result is containing("message") %}{{result.message}}{% endif %} {%if result.code%}[code: {{result.code}}]{%endif%}
  {% endif %}
  {% endfor %}

</div>
</details>
