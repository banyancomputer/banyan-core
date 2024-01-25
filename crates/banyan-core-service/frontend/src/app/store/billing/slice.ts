import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { getInvoiceById, getInvoices, getSubscriptionById, getSubscriptions, subscribe } from "./actions";
import { Invoice, Subscription } from "@/entities/billing";

interface BillingState {
    invoices: Invoice[];
    selectedInvoice: Invoice | null;
    subscriptions: Subscription[];
    selectedSubscription: Subscription | null;
};

const initialState: BillingState = {
    invoices: [],
    subscriptions: [],
    selectedInvoice: null,
    selectedSubscription: null
}

const billingSlice = createSlice({
    name: 'billing',
    initialState,
    reducers: {
        selectInvoice(state, action: PayloadAction<Invoice | null>) {
            state.selectedInvoice = action.payload;
        }
    },
    extraReducers(builder) {
        builder.addCase(getInvoices.fulfilled, (state, action) => {
            state.invoices = action.payload;
        });
        builder.addCase(getInvoiceById.fulfilled, (state, action) => {
            state.selectedInvoice = action.payload;
        });
        builder.addCase(getSubscriptions.fulfilled, (state, action) => {
            const selectedSubscription = action.payload.find(subscription => subscription.currently_active);
            state.subscriptions = action.payload;

            if(selectedSubscription) {
                state.selectedSubscription = selectedSubscription;
            };
        });
        builder.addCase(getSubscriptionById.fulfilled, (state, action) => {
            state.selectedSubscription = action.payload;
        });
    }
});

export const { selectInvoice } = billingSlice.actions;
export default billingSlice.reducer;
