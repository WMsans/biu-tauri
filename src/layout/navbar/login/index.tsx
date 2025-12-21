import React from "react";
import { useTranslation } from "react-i18next";

import { Divider, Modal, ModalBody, ModalContent, Tab, Tabs } from "@heroui/react";

import CodeLogin from "./code-login";
import PasswordLogin from "./password-login";
import QrcodeLogin from "./qrcode-login";

interface Props {
  isOpen: boolean;
  onOpenChange: (isOpen: boolean) => void;
}

const Login = ({ isOpen, onOpenChange }: Props) => {
  const { t } = useTranslation();
  const onClose = () => onOpenChange(false);

  return (
    <Modal size="2xl" isOpen={isOpen} isDismissable={false} onOpenChange={onOpenChange}>
      <ModalContent>
        <ModalBody className="flex-row items-center justify-center gap-8 py-8">
          <QrcodeLogin onClose={onClose} />
          <Divider className="h-42" orientation="vertical" />
          <div className="w-[320px]">
            <Tabs
              aria-label={t("layout.navbar.login.index.")}
              classNames={{ tabContent: "text-lg font-medium mb-4" }}
              fullWidth
              size="lg"
              variant="underlined"
            >
              <Tab key="code" title={t("layout.navbar.login.index..1")}>
                <CodeLogin onClose={onClose} />
              </Tab>
              <Tab key="password" title={t("layout.navbar.login.index..2")}>
                <PasswordLogin onClose={onClose} />
              </Tab>
            </Tabs>
          </div>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
};

export default Login;
