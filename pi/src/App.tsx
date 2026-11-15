import {
  useSettings,
  useStreamDeck,
} from "@mikanseilaboratory/streamdeck-pi-client";
import { useId } from "react";
import type { InputHTMLAttributes } from "react";
import type { ActionMeta } from "./generated/actions";
import { ACTIONS, METHODS, NO_BODY_METHODS } from "./generated/actions";
import type { ActionSettings } from "./generated/contracts";

type Settings = ActionSettings;

const DEFAULTS: Settings = {
  method: null,
  url_press: "",
  url_release: "",
  url_left: "",
  url_right: "",
  url_push: "",
  url_touch: "",
  body: "",
  basic_auth_id: "",
  basic_auth_password: "",
  auth_header: "",
  show_ok: true,
  show_alert: true,
};

const FALLBACK: ActionMeta = {
  uuid: "",
  method: null,
  controller: "keypad",
  legacy: true,
};

export function App() {
  const deck = useStreamDeck();
  const action = deck.actionInfo?.action ?? "";
  const meta = ACTIONS.find((candidate) => candidate.uuid === action) ?? FALLBACK;
  const { settings, setSettings, bind } = useSettings<Settings>(DEFAULTS);

  const method = meta.legacy
    ? settings.method?.trim().toUpperCase() || "GET"
    : meta.method ?? "GET";
  const isDial = meta.controller === "encoder";
  const showBody = !NO_BODY_METHODS.includes(method);

  const update = (patch: Partial<Settings>) => {
    setSettings((previous) => ({ ...previous, ...patch }));
  };

  return (
    <div className="sdpi-wrapper">
      {meta.legacy && (
        <div className="sdpi-item">
          <div className="sdpi-item-label">METHOD</div>
          <select
            className="sdpi-item-value select"
            value={method}
            onChange={(event) => update({ method: event.target.value })}
          >
            {METHODS.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </div>
      )}

      {isDial ? (
        <>
          <TextField label="URL(left)" placeholder="https://example.com/left" {...bind("url_left")} />
          <TextField label="URL(right)" placeholder="https://example.com/right" {...bind("url_right")} />
          <TextField label="URL(push)" placeholder="https://example.com/push" {...bind("url_push")} />
          <TextField label="URL(release)" placeholder="https://example.com/release" {...bind("url_release")} />
          <TextField label="URL(touch)" placeholder="https://example.com/touch" {...bind("url_touch")} />
        </>
      ) : (
        <>
          <TextField label="URL (Press)" placeholder="https://example.com/press" {...bind("url_press")} />
          <TextField label="URL (Release)" placeholder="https://example.com/release" {...bind("url_release")} />
        </>
      )}

      {showBody && (
        <div className="sdpi-item">
          <div className="sdpi-item-label">BODY</div>
          <textarea
            className="sdpi-item-value"
            rows={3}
            value={settings.body}
            placeholder='{"key":"value"}'
            onChange={(event) => update({ body: event.target.value })}
          />
        </div>
      )}

      <TextField label="Basic AUTH ID" {...bind("basic_auth_id")} />
      <TextField label="Basic AUTH Password" type="password" {...bind("basic_auth_password")} />
      <TextField label="Authorization Header" {...bind("auth_header")} />

      <CheckField
        label='Enable "OK" Notification'
        checked={settings.show_ok}
        onChange={(checked) => update({ show_ok: checked })}
      />
      <CheckField
        label='Enable "WARNING" Notification'
        checked={settings.show_alert}
        onChange={(checked) => update({ show_alert: checked })}
      />
    </div>
  );
}

type TextFieldProps = {
  label: string;
} & InputHTMLAttributes<HTMLInputElement>;

function TextField({ label, ...props }: TextFieldProps) {
  return (
    <div className="sdpi-item">
      <div className="sdpi-item-label">{label}</div>
      <input className="sdpi-item-value" {...props} />
    </div>
  );
}

function CheckField({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  const id = useId();
  return (
    <div className="sdpi-item" {...{ type: "checkbox" }}>
      <div className="sdpi-item-label">{label}</div>
      <div className="sdpi-item-value">
        <input
          id={id}
          type="checkbox"
          checked={checked}
          onChange={(event) => onChange(event.target.checked)}
        />
        <label htmlFor={id}>
          <span />
        </label>
      </div>
    </div>
  );
}
