import re

import requests
from bs4 import BeautifulSoup, NavigableString

registered_vendor_ids = {}
url = "https://docs.microsoft.com/en-us/typography/vendors/"
outfile = "profile-googlefonts/resources/vendor_ids.txt"
response = requests.get(url)
response.raise_for_status()
content = response.text
# Strip all <A> HTML tags from the raw HTML. The current page contains a
# closing </A> for which no opening <A> is present, which causes
# beautifulsoup to silently stop processing that section from the error
# onwards. We're not using the href's anyway.
content = re.sub("<a[^>]*>", "", content, flags=re.IGNORECASE)
content = re.sub("</a>", "", content, flags=re.IGNORECASE)
soup = BeautifulSoup(content, "html.parser")

IDs = [chr(c + ord("a")) for c in range(ord("z") - ord("a") + 1)]
IDs.append("0-9-")

for section_id in IDs:
    section = soup.find("h2", {"id": section_id})
    if not section:
        continue

    table = section.find_next_sibling("table")
    if not table or isinstance(table, NavigableString):
        continue

    # print ("table: '{}'".format(table))
    for row in table.findAll("tr"):
        # print("ROW: '{}'".format(row))
        cells = row.findAll("td")
        if not cells:
            continue

        labels = list(cells[1].stripped_strings)

        # pad the code to make sure it is a 4 char string,
        # otherwise eg "CF  " will not be matched to "CF"
        code = cells[0].string.strip()
        code = code + (4 - len(code)) * " "
        registered_vendor_ids[code] = labels[0]

        # Do the same with NULL-padding:
        code = cells[0].string.strip()
        code = code + (4 - len(code)) * chr(0)
        registered_vendor_ids[code] = labels[0]

with open(outfile, "w", encoding="utf-8") as fh:
    for vendor in registered_vendor_ids.keys():
        print(vendor, file=fh)
