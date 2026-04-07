import { useEffect, useState } from "react";

const InstallPWA = () => {
  const [supportsPWA, setSupportsPWA] = useState(false);
  const [promptInstall, setPromptInstall] = useState(null);

  useEffect(() => {
    const handler = (e) => {
      e.preventDefault();
      setSupportsPWA(true);
      setPromptInstall(e);
    };
    window.addEventListener("beforeinstallprompt", handler);

    return () => window.removeEventListener("beforeinstallprompt", handler);
  }, []);

  const onClick = (evt) => {
    evt.preventDefault();
    if (!supportsPWA) {
      alert(
        "This browser does not support installing progress web apps. Try Chrome or Edge",
      );
    }

    if (!promptInstall) {
      return;
    }
    promptInstall.prompt();
  };

  return (
    <button
      className="scs-button-secondary w-full"
      id="setup_button"
      aria-label="Install app"
      title="Install app"
      onClick={onClick}
    >
      Install App
    </button>
  );
};

export default InstallPWA;
