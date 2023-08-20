import React from "react";
import { cn } from "../utils/cn";
import { useIntl } from "react-intl";

export const LoadingSpinner = ({ className }: { className?: string }) => {
  const intl = useIntl();
  return (
    <div
      className={cn(
        "inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-gray-200 border-r-blue-800 align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]",
        className
      )}
      role="status"
    >
      <span
        className={cn(
          "!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]"
        )}
      >
        {intl.formatMessage({ id: "name" })}...
      </span>
      <FormattedMessage id="name"></FormattedMessage>
      <FormattedMessage id={"name"} />
    </div>
  );
};

export const FullPageLoadingSpinner = () => (
  <div className="flex flex-1 items-center justify-center">
    <LoadingSpinner className="h-16 w-16 border-[6px]" />
  </div>
);
