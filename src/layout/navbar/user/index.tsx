import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router";

import {
  Dropdown,
  DropdownTrigger,
  DropdownMenu,
  DropdownItem,
  Avatar,
  useDisclosure,
  addToast,
  type DropdownItemProps,
} from "@heroui/react";
import {
  RiExternalLinkLine,
  RiFeedbackLine,
  RiLoginCircleLine,
  RiLogoutCircleLine,
  RiProfileLine,
  RiSettings3Line,
} from "@remixicon/react";

import ConfirmModal from "@/components/confirm-modal";
import { postPassportLoginExit } from "@/service/passport-login-exit";
import { useSettings } from "@/store/settings";
import { useToken } from "@/store/token";
import { useUser } from "@/store/user";

import Login from "../login";

const UserCard = () => {
  const { t } = useTranslation();
  const user = useUser(s => s.user);
  const clearUser = useUser(s => s.clear);
  const clearToken = useToken(s => s.clear);
  const navigate = useNavigate();
  const updateSettings = useSettings(s => s.update);

  const { isOpen: isLoginModalOpen, onOpen: openLoginModal, onOpenChange: onLoginModalOpenChange } = useDisclosure();

  const {
    isOpen: isConfirmLogoutModalOpen,
    onOpen: openConfirmLogoutModal,
    onOpenChange: onConfirmLogoutModalOpenChange,
  } = useDisclosure();

  const logout = async () => {
    const csrfToken = await window.electron.getCookie("bili_jct");
    if (!csrfToken) {
      addToast({
        title: t("layout.navbar.user.index.csrf-token"),
        color: "danger",
      });
      return false;
    }

    const res = await postPassportLoginExit({
      biliCSRF: csrfToken,
    });
    if (res?.code === 0) {
      clearToken();
      clearUser();
      updateSettings({
        hiddenMenuKeys: [],
      });
      navigate("/");
      return true;
    } else {
      addToast({
        title: res?.message || "退出登录失败",
        color: "danger",
      });
      return false;
    }
  };

  const dropdownItems: (DropdownItemProps & { label: string; hidden?: boolean })[] = [
    {
      key: "login",
      label: t("layout.navbar.login.code-login..3"),
      startContent: <RiLoginCircleLine size={18} />,
      hidden: user?.isLogin,
      onPress: openLoginModal,
    },
    {
      key: "profile",
      label: t("layout.navbar.user.index..2"),
      startContent: <RiProfileLine size={18} />,
      hidden: !user?.isLogin,
      onPress: () => navigate(`/user/${user?.mid}`),
    },
    {
      key: "settings",
      label: t("pages.settings.index."),
      startContent: <RiSettings3Line size={18} />,
      onPress: () => navigate("/settings"),
    },
    {
      key: "feedback",
      label: t("components.error-fallback.index."),
      startContent: <RiFeedbackLine size={18} />,
      endContent: <RiExternalLinkLine size={18} />,
      onPress: () => window.electron.openExternal("https://github.com/wood3n/biu/issues"),
    },
    {
      key: "logout",
      label: t("layout.navbar.user.index..3"),
      startContent: <RiLogoutCircleLine size={18} />,
      color: "danger" as const,
      className: "text-danger",
      hidden: !user?.isLogin,
      onPress: openConfirmLogoutModal,
    },
  ].filter(item => !item.hidden);

  return (
    <>
      <Dropdown
        shouldBlockScroll={false}
        classNames={{
          content: "min-w-[140px]",
        }}
      >
        <DropdownTrigger>
          <Avatar
            isBordered
            showFallback
            size="sm"
            as="button"
            type="button"
            className="mr-4 cursor-pointer transition-transform hover:scale-105"
            src={user?.face}
          />
        </DropdownTrigger>
        <DropdownMenu aria-label={t("layout.navbar.user.index.")} variant="flat" items={dropdownItems}>
          {({ key, label, ...rest }) => (
            <DropdownItem key={key} {...rest}>
              {label}
            </DropdownItem>
          )}
        </DropdownMenu>
      </Dropdown>
      <ConfirmModal
        type="danger"
        title={t("layout.navbar.user.index..1")}
        isOpen={isConfirmLogoutModalOpen}
        onOpenChange={onConfirmLogoutModalOpenChange}
        onConfirm={logout}
      />

      <Login isOpen={isLoginModalOpen} onOpenChange={onLoginModalOpenChange} />
    </>
  );
};

export default UserCard;
