import type { FallbackProps } from "react-error-boundary";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router";

import { Button } from "@heroui/react";

import { ReactComponent as ErrorIllustration } from "@/assets/images/error.svg";

const Fallback = ({ resetErrorBoundary }: FallbackProps) => {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <div className="window-drag bg-content1 flex h-screen w-screen flex-col items-center justify-center space-y-4">
      <ErrorIllustration style={{ width: 480 }} />
      <div className="window-no-drag flex items-center space-x-2">
        <Button onPress={() => window.electron.openExternal("https://github.com/wood3n/biu/issues")}>
          {t("components.error-fallback.index.")}
        </Button>
        <Button
          color="primary"
          onPress={() => {
            navigate("/");
            resetErrorBoundary();
          }}
        >
          {t("components.error-fallback.index..1")}
        </Button>
      </div>
    </div>
  );
};
export default Fallback;
