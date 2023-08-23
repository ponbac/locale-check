import React from "react";
import { FormattedMessage, useIntl } from "react-intl";
import { RamiButton, TableRowModal, cn, useToast, useZodForm } from "ui";
import {
  ProductEmissionDto,
  useGetAdministrationFuels,
  usePostAdministrationProductEmissions,
  usePutAdministrationProductEmissionsId,
} from "../../../api/sustainabilityAdministrationServiceClient";
import { z } from "zod";
import LabeledInput from "ui/components/LabeledInput";
import { ParsedAPIErrorMessage } from "shell/app-shell/AppError";
import { AxiosError } from "axios";
import { SharedResultsAndErrorsIError } from "../../../api/adminModuleApi";

const AddProductSchema = z.object({
  articleNumber: z.string().nonempty(),
  productDescription: z.string().nonempty(),
  pimId: z
    .string()
    .nullish()
    .transform((val) => (val === "" ? null : val)),
  dailyUsage: z.coerce.number().min(0),
  hourlyConsumption: z.coerce.number().min(0),
  fuelId: z.string().nonempty(),
});

const CO2ProductModal = (props: {
  open: boolean;
  onClose: () => void;
  onMutate: () => void;
  activeRow: ProductEmissionDto | null;
}) => {
  const intl = useIntl();
  const { toast } = useToast();

  const modalMode = props.activeRow === null ? "add" : "edit";

  const { data: fuels } = useGetAdministrationFuels();

  const methods = useZodForm({
    schema: AddProductSchema,
    defaultValues: {
      articleNumber: props.activeRow?.articleNumber ?? "",
      productDescription: props.activeRow?.productDescription ?? "",
      pimId: props.activeRow?.pimId ?? "",
      dailyUsage: props.activeRow?.dailyUsage ?? 0,
      hourlyConsumption: props.activeRow?.hourlyConsumption ?? 0,
      fuelId: props.activeRow?.fuelId ?? fuels?.at(0)?.id ?? "",
    },
  });

  const handleClose = () => {
    props.onClose();
    methods.reset();
  };

  const onMutationSuccess = () => {
    toast({
      title: intl.formatMessage({
        id: "sustainability_admin.mutate_product_success_toast_title",
      }),
      description: intl.formatMessage({
        id: "sustainability_admin.mutate_product_success_toast_body",
      }),
      variant: "success",
      duration: 5000,
    });
    props.onMutate();
    handleClose();
  };

  const onMutationError = (
    error: AxiosError<SharedResultsAndErrorsIError[]>
  ) => {
    toast({
      title: intl.formatMessage({
        id: "sustainability_admin.edit_factors_error_toast_title",
      }),
      description: (
        <div>
          <h2>
            {intl.formatMessage({
              id: "sustainability_admin.edit_factors_error_toast_body",
            })}
            :
          </h2>
          <ParsedAPIErrorMessage error={error} />
        </div>
      ),
      variant: "destructive",
      duration: 15000,
    });
  };

  const { mutate: addCO2Product } = usePostAdministrationProductEmissions({
    mutation: {
      onSuccess: onMutationSuccess,
      onError: onMutationError,
    },
  });

  const { mutate: editCO2Product } = usePutAdministrationProductEmissionsId({
    mutation: {
      onSuccess: onMutationSuccess,
      onError: onMutationError,
    },
  });

  return (
    <TableRowModal
      open={props.open}
      onClose={handleClose}
      className="h-fit overflow-visible"
      header={
        <div className={cn("flex gap-4")}>
          <div>
            {
              <FormattedMessage
                id={
                  modalMode === "add"
                    ? "sustainability_admin.add_product"
                    : "sustainability_admin.edit_product"
                }
                values={{ sub: (chunk) => <sub>{chunk}</sub> }}
              />
            }
          </div>
        </div>
      }
    >
      <form
        onSubmit={methods.handleSubmit((data) => {
          if (modalMode === "add") {
            addCO2Product({
              data,
            });
          } else if (modalMode === "edit") {
            editCO2Product({
              id: props.activeRow?.id ?? "",
              data,
            });
          }
        })}
        className="flex h-full flex-col justify-between"
      >
        <div className={cn("flex flex-col gap-2")}>
          <LabeledInput
            required
            label={intl.formatMessage({ id: "common.article_number" })}
            {...methods.register("articleNumber")}
          />
          <LabeledInput
            required
            label={intl.formatMessage({ id: "common.product_description" })}
            {...methods.register("productDescription")}
          />
          <LabeledInput
            label={intl.formatMessage({ id: "sustainability_admin.pim_id" })}
            {...methods.register("pimId")}
          />
          <LabeledInput
            required
            label={intl.formatMessage({
              id: "sustainability_admin.daily_usage",
            })}
            type="number"
            min={0}
            step="any"
            {...methods.register("dailyUsage")}
          />
          <LabeledInput
            required
            label={intl.formatMessage({
              id: "sustainability_admin.hourly_consumption",
            })}
            type="number"
            min={0}
            step="any"
            {...methods.register("hourlyConsumption")}
          />
          <div className={"flex flex-col"}>
            <label
              className={cn("text-sm font-semibold")}
              htmlFor={"fuel-select"}
            >
              {intl.formatMessage({
                id: "sustainability_admin.fuel_id",
              })}{" "}
              *
            </label>
            <select id="fuel-select" {...methods.register("fuelId")}>
              {fuels?.map((fuel) => (
                <option key={fuel.id} value={fuel.id ?? undefined}>
                  {fuel.name}
                </option>
              ))}
            </select>
          </div>
        </div>
        <div className={cn("flex justify-center gap-4 pt-16")}>
          <RamiButton
            onClick={handleClose}
            variant="outlined light"
            size="extra small"
          >
            {" "}
            <FormattedMessage id={"common.cancel"} />
          </RamiButton>
          <RamiButton type="submit" size="extra small">
            {" "}
            <FormattedMessage id={"common.save"} />
          </RamiButton>
        </div>
      </form>
    </TableRowModal>
  );
};

export default CO2ProductModal;
