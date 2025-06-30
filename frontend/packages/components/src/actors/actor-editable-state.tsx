import { Button, ScrollArea, WithTooltip } from "@rivet-gg/components";
import { faRotateLeft, faSave, Icon } from "@rivet-gg/icons";
import { useState } from "react";
import { ActorStateChangeIndicator } from "./actor-state-change-indicator";
import { Json } from "../json";
import {
  ActorId,
  useActorStatePatchMutation,
  useActorStateStream,
} from "./queries";

const isValidJson = (json: string | null): json is string => {
  if (!json) return false;
  try {
    JSON.parse(json);
    return true;
  } catch {
    return false;
  }
};

interface ActorEditableStateProps {
  actorId: ActorId;
  state: unknown;
}

export function ActorEditableState({
  actorId,
  state,
}: ActorEditableStateProps) {
  const [value, setValue] = useState<string | null>(null);

  const isValid = isValidJson(value) ? JSON.parse(value) : false;

  const { mutate } = useActorStatePatchMutation(actorId);

  useActorStateStream(actorId);

  return (
    <>
      <div className="flex justify-between items-center border-b gap-1 p-2">
        <div className="flex items-center justify-start gap-1">
          <ActorStateChangeIndicator state={state} />
        </div>
        <div className="flex gap-2">
          <WithTooltip
            content="Save state"
            trigger={
              <Button
                size="icon-sm"
                variant="outline"
                disabled={!isValid}
                onClick={() => {
                  setValue(null);
                }}
              >
                <Icon icon={faSave} />
              </Button>
            }
          />
          <WithTooltip
            content="Restore original state"
            trigger={
              <Button
                size="icon-sm"
                variant="outline"
                onClick={() => {
                  setValue(null);
                }}
              >
                <Icon icon={faRotateLeft} />
              </Button>
            }
          />
        </div>
      </div>
      <div className="flex flex-1 min-h-0 w-full">
        <ScrollArea className="w-full h-full">
          <Json
            value={state}
            onChange={(updater) => {
              mutate(
                typeof updater === "function"
                  ? updater(structuredClone(state) as Record<string, unknown>)
                  : updater
              );
            }}
          />
        </ScrollArea>
      </div>
    </>
  );
}
