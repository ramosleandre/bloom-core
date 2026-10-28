"""Export the actual core and D transport contracts; no handwritten TypeScript copies."""

import json

from pydantic import TypeAdapter

from bloom.interface.bridge import (
    ApprovalCommand,
    Authenticate,
    ChoiceCommand,
    ErrorEvent,
    StartCommand,
    StateEvent,
)

schema = TypeAdapter(
    StateEvent | ErrorEvent | Authenticate | StartCommand | ChoiceCommand | ApprovalCommand
).json_schema(mode="serialization")
# The wire format uses model_dump_json(exclude_unset=False): defaults are always present.
for definition in schema.get("$defs", {}).values():
    if definition.get("type") == "object" and "properties" in definition:
        definition["required"] = list(definition["properties"])
print(json.dumps(schema))
